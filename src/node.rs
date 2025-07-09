/// Simulation of participating nodes
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::sync::mpsc::{Receiver, Sender, channel};

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

#[derive(Debug, Default)]
pub(crate) struct Message<V> {
    src: NodeId,
    dst: NodeId,
    cookie: Cookie,
    msg: MessageKind<V>,
}

#[derive(Debug, Default)]
pub(crate) enum MessageKind<V> {
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
pub(crate) struct Node<V> {
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
    active_connections: HashMap<Cookie, Message<V>>,
    /// connection to network, if any
    pub(crate) sender: Option<Sender<Message<V>>>,
    pub(crate) receiver: Option<Receiver<Message<V>>>,
}

impl<V: Default> Node<V> {
    pub(crate) fn new() -> Self {
        Self {
            id: NodeId::new(),
            ..Default::default()
        }
    }
}

impl<V> Node<V> {
    pub fn get_id(&self) -> NodeId {
        self.id
    }
    fn handle_message(&mut self, msg: Message<V>) {
        todo!()
    }

    pub(crate) fn send_message(&self, dst: NodeId, msg: MessageKind<V>) {
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

    fn receive_message(&self) {
        if let Some(rx) = &self.receiver {
            while let Ok(msg) = rx.recv() {
                if msg.dst != self.id {
                    // get a message for another node
                    continue;
                }
                todo!()
            }
        }
    }

    fn find_node(&mut self, id: NodeId) {
        todo!()
    }

    fn store(&mut self, key: u128, value: V) {
        todo!()
    }
    fn handle_communication(&mut self, msg: &Message<V>) {
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
