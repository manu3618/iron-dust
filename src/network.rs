/// Simulation of network
///
///
use crate::node::{Message, Node, NodeId, Payload};
use rand::seq::IteratorRandom;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{Receiver, Sender, channel};

#[derive(Debug)]
pub(crate) struct Network<V: std::clone::Clone + PartialEq + Eq> {
    nodes: HashMap<NodeId, Arc<Mutex<Node<V>>>>,
    messages: VecDeque<Message<V>>,

    /// message channel, from node to network
    node_receiver: Receiver<Message<V>>,
    node_sender: Sender<Message<V>>,
}

impl<V: std::fmt::Debug + Default + std::clone::Clone + Eq + PartialEq> Network<V> {
    pub fn new() -> Self {
        let (tx, rx) = channel(16);
        Self {
            nodes: Default::default(),
            messages: Default::default(),
            node_receiver: rx,
            node_sender: tx,
        }
    }

    /// Add a node to the current network
    pub fn add_node(&mut self) -> NodeId {
        let mut new_node = Node::new();
        let new_id = new_node.get_id();

        new_node.sender = Some(self.node_sender.clone());
        new_node.receiver = Some(self.node_sender.subscribe());

        if let Some(&dst) = self.nodes.keys().choose(&mut rand::rng()) {
            new_node.send_message(dst, Payload::FindNode(new_id))
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

impl<V: std::clone::Clone + Eq + PartialEq> Network<V> {
    /// Randomly select a node and ask this node to retrieve the value
    pub fn get_value(&self, key: u128) -> Option<V> {
        todo!()
    }

    /// Randomly select a node and kill it
    pub fn kill_node(&self) {
        todo!()
    }
}
