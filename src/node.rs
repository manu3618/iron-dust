/// Simulation of participating nodes
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::time::{Duration, Instant, timeout_at};

/// Maximum number of known neighbors.
/// The default behavior is to keep only the most recent one
static MAX_NEIGHBORS: usize = 8;
static BUCKET_SIZE: usize = 4;

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
struct Cookie(u128);

impl Cookie {
    fn new() -> Self {
        let mut rng = rand::rng();
        Self(rng.random())
    }
}

impl From<u128> for Cookie {
    fn from(item: u128) -> Self {
        Self(item)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
pub(crate) struct NodeId(u128);

impl From<u128> for NodeId {
    fn from(item: u128) -> Self {
        Self(item)
    }
}

impl NodeId {
    fn new() -> Self {
        let mut rng = rand::rng();
        Self(rng.random())
    }
    fn dist(&self, other: &Self) -> u128 {
        self.0 ^ other.0
    }

    fn get_bucket(&self, other: &Self) -> usize {
        (self.dist(other) * (u128::MAX / BUCKET_SIZE as u128)) as usize
    }

    /// Get on which bucket the other shlud belong
    ///
    /// Given an Id, other Ids should go in bucket sush that each bucket
    /// contains at most BUCKET_SIZE nodes. The number of bucket is at most
    /// BUCKET_SIZE.
    /// This mechanism ensure that each node knows other nodes far away and
    /// near itself.
    fn to_buckets(&self, list: Vec<NodeId>) -> Vec<Vec<NodeId>> {
        let mut buckets = vec![Vec::new(); BUCKET_SIZE];
        for id in list {
            buckets[self.get_bucket(&id)].push(id)
        }
        buckets
            .iter()
            .map(|v| v[v.len().max(BUCKET_SIZE) - BUCKET_SIZE..].to_vec())
            .collect()
    }

    /// Remove exceeding IDs from the list and returns removed IDs
    fn truncate_list(&self, list: &mut Vec<NodeId>) -> Vec<NodeId> {
        let init_list: Vec<NodeId> = list.clone();
        *list = self
            .to_buckets(init_list.clone())
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        list.iter().filter(|a| !list.contains(a)).copied().collect()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Message<V: std::clone::Clone + PartialEq> {
    src: NodeId,
    dst: NodeId,
    cookie: Cookie,
    msg: Payload<V>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) enum Payload<V: std::clone::Clone> {
    #[default]
    Ping,
    Store {
        key: u128,
        value: V,
    },
    FindNode(NodeId),
    /// Answer to FindNode
    KnownNodes(Vec<NodeId>),
    FindValue(u128),
}

impl<V: std::clone::Clone + PartialEq + Default> Message<V> {
    pub fn new() -> Self {
        Self {
            cookie: Cookie::new(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Node<V: std::clone::Clone + Eq + PartialEq> {
    id: NodeId,
    /// local part of the DHT
    data: HashMap<u128, V>,
    /// Message to be processed
    waiting_messages: Arc<Mutex<HashMap<u128, VecDeque<Message<V>>>>>,
    /// Last neighbors seen
    recent: Arc<Mutex<Vec<NodeId>>>,
    /// IDs of known neighbors
    neighbors: Arc<Mutex<Vec<NodeId>>>,
    /// state of connections being served
    active_connections: Arc<Mutex<HashMap<Cookie, Vec<Message<V>>>>>,
    /// connection to network, if any
    pub(crate) sender: Option<Sender<Message<V>>>,
    pub(crate) receiver: Option<Receiver<Message<V>>>,
}

impl<V: Default + std::clone::Clone + PartialEq + Eq> Node<V> {
    pub(crate) fn new() -> Self {
        Self {
            id: NodeId::new(),
            ..Default::default()
        }
    }
}

impl<V: std::clone::Clone + Eq + PartialEq> Node<V> {
    pub fn get_id(&self) -> NodeId {
        self.id
    }

    /// handle all messages queued for now
    ///
    async fn drain_messages(&mut self) {
        // self.active_connections may be modified during processing of to_treat
        let to_treat = Arc::clone(&self.active_connections);
        let mut active_connections = to_treat.lock().await;
        let mut to_reinsert = HashMap::new();
        for (cookie, messages) in active_connections.drain() {
            for message in messages {
                // may modify self.active_connections
                if !self.handle_message(&message).await.is_ok() {
                    // this message should be reinserted
                    to_reinsert
                        .entry(cookie)
                        .or_insert(Vec::new())
                        .push(message)
                }
            }
        }

        // reinsert skipped messages
        for (cookie, mut messages) in to_reinsert {
            active_connections
                .entry(cookie)
                .or_insert(Vec::new())
                .append(&mut messages)
        }
    }

    /// Handle a single message
    ///
    /// Return Ok(()) if the message can be handled immediately
    async fn handle_message(&mut self, msg: &Message<V>) -> Result<(), String> {
        {
            let recent = Arc::clone(&self.recent);
            let mut recent = recent.lock().await;
            recent.push(msg.src);
            let rejected = self.id.truncate_list(&mut recent);
        }
        match msg.msg {
            Payload::Ping => todo!(),
            Payload::Store { .. } => todo!(),
            Payload::FindNode(_) => todo!(),
            Payload::KnownNodes(_) => todo!(),
            Payload::FindValue(_) => todo!(),
        }
    }

    pub(crate) fn send_message(&self, dst: NodeId, msg: Payload<V>) {
        let message = Message {
            src: self.get_id(),
            dst,
            cookie: Cookie::new(),
            msg,
        };
        if let Some(tx) = &self.sender {
            tx.send(message);
        }
    }

    /// Get the response for the message msg
    async fn get_payload_response(
        &mut self,
        dst: NodeId,
        payload: Payload<V>,
    ) -> Option<Payload<V>> {
        let msg = Message {
            src: self.get_id(),
            dst,
            cookie: Cookie::new(),
            msg: payload,
        };
        self.get_message_response(msg).await.map(|m| m.msg)
    }

    /// Get the response for the message msg
    async fn get_message_response(&self, msg: Message<V>) -> Option<Message<V>> {
        let mut receiver = self.sender.clone()?.subscribe();
        let cookie = msg.cookie.clone();
        let _ = self.sender.clone()?.send(msg.clone());
        let conn = Arc::clone(&self.active_connections);
        let mut active_connections = conn.lock().await;
        while let Ok(Ok(resp)) =
            timeout_at(Instant::now() + Duration::from_secs(10), receiver.recv()).await
        {
            if resp == msg {
                continue;
            }
            if cookie == resp.cookie {
                return Some(resp);
            }
            active_connections
                .entry(cookie.clone())
                .or_insert(Vec::new())
                .push(resp)
        }
        active_connections.get_mut(&cookie)?.pop()
    }

    async fn answer_find_node(&self, msg: Message<V>) {
        if let Payload::FindNode(s) = msg.msg {
            let known_nodes = self.get_nearest_nodes(s).await;
            let ans = Message {
                src: msg.dst,
                dst: msg.src,
                cookie: msg.cookie,
                msg: Payload::KnownNodes(known_nodes),
            };
            if let Some(tx) = &self.sender {
                tx.send(ans);
            }
        }
    }

    /// Get the k known nearest nodes
    async fn get_nearest_nodes(&self, id: NodeId) -> Vec<NodeId> {
        let k = 3;
        let recent = Arc::clone(&self.recent);
        let mut recent = recent.lock().await;
        let mut nearest = recent.iter().cloned().collect::<Vec<_>>();
        nearest.sort_by(|a, b| a.dist(&id).cmp(&b.dist(&id)));
        nearest.truncate(k);
        nearest
    }

    /// Handle single FindNode message
    ///
    /// id: node to find
    /// dst: node asked
    async fn find_node(&mut self, id: NodeId, dst: NodeId) {
        let payload = Payload::FindNode(id);
        let message = Message {
            src: self.get_id(),
            dst,
            cookie: Cookie::new(),
            msg: Payload::<V>::FindNode(id),
        };

        let r = self.get_payload_response(dst, payload).await;
        todo!()
    }

    /// Look up for a node
    async fn lookup_node(&self, id: NodeId) {
        // TODO: to review
        let alpha = 3;
        let recent = Arc::new(self.recent);
        let mut recent = recent.lock().await;
        recent.sort_by_key(|&x| x.dist(&self.id));
        let current_best = recent.first().expect("at least one known node");
        let handlers = Vec::new();
        for &n in recent.iter().take(alpha) {
            handlers.push(tokio::spawn(self.find_node(id, n.clone())));
            todo!();
        }
    }

    pub async fn store(&mut self, key: u128, value: V) {
        if let Some(v) = self.data.get_mut(&key) {
            todo!()
        }

        // Update nearest nodes
        self.lookup_node(key.into()).await;

        // Duplicate data
        for dst in self.get_nearest_nodes(key.into()) {
            let msg = Payload::Store {
                key: key.into(),
                value: value.clone(),
            };
            self.send_message(dst, msg);
        }
        self.data.insert(key, value);
    }

    pub async fn get_value(&mut self, key: u128) -> Option<V> {
        todo!()
    }
}

// fn requeue<T>(v: &mut VecDeque<T>, value: T, max_size: usize)
// where
//     T: PartialEq,
// {
//     if let Some(idx) = v.iter().position(|x| *x == value) {
//         v.remove(idx);
//     }
//     v.push_back(value);
//     while v.len() > max_size {
//         v.pop_front();
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_nodeid_distance() {
        for _ in 0..100 {
            let a = NodeId::new();
            let b = NodeId::new();
            let c = NodeId::new();
            // a.dist(&b) >= 0 (u128)
            assert_eq!(a.dist(&a), 0);
            assert_eq!(a.dist(&b), b.dist(&a));
            if a.dist(&c).overflowing_add(c.dist(&b)).1 {
                // dist(a, b) <= u128::MAX < dist(a, b) + dist(b, c)
            } else {
                assert!(a.dist(&b) <= a.dist(&c) + c.dist(&b));
            }
        }
    }

    #[tokio::test]
    async fn test_answer() {
        assert!(true);
    }
}
