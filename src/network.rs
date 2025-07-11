/// Simulation of network
///
///
use crate::node::{Message, MessageKind, Node, NodeId};
use rand::seq::IteratorRandom;
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct Network<V> {
    nodes: HashMap<NodeId, Arc<Mutex<Node<V>>>>,
    messages: VecDeque<Message<V>>,

    /// message channel, from node to network
    net_receiver: Receiver<Message<V>>,
    net_sender: Sender<Message<V>>,

    /// message channels, from network to node
    node_sender: Vec<Sender<Message<V>>>,
}

impl<V: std::fmt::Debug + Default> Network<V> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            nodes: Default::default(),
            messages: Default::default(),
            net_receiver: rx,
            net_sender: tx.clone(),
            node_sender: Vec::new(),
        }
    }

    /// Add a node to the current network
    pub fn add_node(&mut self) -> NodeId {
        let mut new_node = Node::new();
        let new_id = new_node.get_id();
        new_node.sender = Some(self.net_sender.clone());
        let (tx, rx) = channel();
        new_node.receiver = Some(rx);
        self.node_sender.push(tx);

        if let Some(&dst) = self.nodes.keys().choose(&mut rand::rng()) {
            new_node.send_message(dst, MessageKind::FindNode(new_id))
        }
        self.nodes.insert(new_id, Arc::new(Mutex::new(new_node)));
        new_id
    }

    /// Randomly select a node and ask this node to insert a value
    pub fn insert_value(&mut self, key: u128, value: V) {
        let node = self
            .nodes
            .values()
            .choose(&mut rand::rng())
            .expect("Network should not be empty");
        {
            let mut n = node.lock().unwrap();
            &n.store(key, value);
        }
    }
}

impl<V> Network<V> {
    /// Randomly select a node and ask this node to retrieve the value
    pub fn get_value(&self, key: u128) -> Option<V> {
        todo!()
    }

    /// Randomly select a node and kill it
    pub fn kill_node(&self) {
        todo!()
    }
}
