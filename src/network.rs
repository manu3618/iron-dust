/// Simulation of network
///
///
use crate::node::{Message, MessageKind, Node};
use rand::Rng;
use rand::seq::IteratorRandom;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Default)]
struct Network<V> {
    nodes: HashMap<u128, Node<V>>,
    messages: VecDeque<Message<V>>,
}

impl<V: Default> Network<V> {
    fn add_node(&mut self) -> u128 {
        let mut new_node = Node::new();
        let new_id = new_node.get_id();
        if let Some(&dst) = self.nodes.keys().choose(&mut rand::rng()) {
            new_node.send_message(dst, MessageKind::FindNode(new_id))
        }
        self.nodes.insert(new_id, new_node);
        new_id
    }
}

impl<V> Network<V> {}
