/// Simulation of participating nodes
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::time::{Duration, Instant, timeout_at};

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
}

#[derive(Debug, Clone, PartialEq)]
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
    FindValue(u128),
}

#[derive(Debug, Default)]
pub(crate) struct Node<V: std::clone::Clone + Eq + PartialEq> {
    id: NodeId,
    /// local part of the DHT
    data: HashMap<u128, V>,
    /// Message to be processed
    waiting_messages: HashMap<u128, VecDeque<Message<V>>>,
    /// Last neighbors seen
    recent: VecDeque<u128>,
    /// IDs of known neighbors
    neighbors: Vec<u128>,
    /// state of connections being served
    active_connections: HashMap<Cookie, Vec<Message<V>>>,
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

    /// await, receive, queue, handle messages.
    async fn receive_messages(&mut self) {
        let mut receiver = self
            .sender
            .clone()
            .expect("connection to network should be set")
            .subscribe();
        while let Ok(resp) = receiver.recv().await {
            let cookie = resp.cookie.clone();
            self.active_connections
                .entry(cookie)
                .or_insert(Vec::new())
                .push(resp);
            self.drain_messages().await;
        }
        todo!()
    }

    /// handle all messages queued for now
    ///
    async fn drain_messages(&mut self) {
        // self.active_connections may be modified during processing of to_treat
        let mut to_treat = self.active_connections.clone();
        let mut to_reinsert = HashMap::new();
        for (cookie, messages) in to_treat.drain() {
            for message in messages {
                // may modify self.active_connections
                if !self.handle_message(&message).is_ok() {
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
            self.active_connections
                .entry(cookie)
                .or_insert(Vec::new())
                .append(&mut messages)
        }
    }

    /// Handle a single message
    ///
    /// Return Ok(()) if the message can be handled immediately
    fn handle_message(&self, msg: &Message<V>) -> Result<(), String> {
        match msg.msg {
            Payload::Ping => todo!(),
            Payload::Store { .. } => todo!(),
            Payload::FindNode(_) => todo!(),
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

    async fn get_response(&mut self, msg: Message<V>) -> Option<Message<V>> {
        let mut receiver = self.sender.clone()?.subscribe();
        let cookie = msg.cookie.clone();
        let _ = self.sender.clone()?.send(msg.clone());
        while let Ok(Ok(resp)) =
            timeout_at(Instant::now() + Duration::from_secs(10), receiver.recv()).await
        {
            if resp == msg {
                continue;
            }
            if cookie == resp.cookie {
                return Some(resp);
            }
            self.active_connections
                .entry(cookie.clone())
                .or_insert(Vec::new())
                .push(resp)
        }
        self.active_connections.get_mut(&cookie)?.pop()
    }

    async fn find_node(&mut self, id: NodeId) {
        let alpha = 3;
        let init_best = self
            .recent
            .iter()
            .map(|&x| NodeId::from(x).dist(&self.id))
            .min();
        todo!()
    }

    pub fn store(&mut self, key: u128, value: V) {
        todo!()
    }
}

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
}
