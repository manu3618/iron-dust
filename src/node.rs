/// Simulation of participating nodes
use rand::Rng;
use std::collections::{HashMap, VecDeque};
use std::fmt::Binary;
use std::hash::Hash;

#[derive(Debug, Default)]
pub(crate) struct Message<V> {
    src: u128,
    dst: u128,
    cookie: String,
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
    FindNode(u128),
    FindValue(u128),
}

#[derive(Debug, Default)]
pub(crate) struct Node<V> {
    id: u128,
    /// local part of the DHT
    data: HashMap<u128, V>,
    /// Message to be processed
    waiting_messages: HashMap<u128, VecDeque<Message<V>>>,
    /// Last neighbors seen
    recent: VecDeque<u128>,
    /// IDs of known neighbors
    neighbors: Vec<u128>,
}

impl<V: Default> Node<V> {
    pub(crate) fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            id: rng.random(),
            ..Default::default()
        }
    }
}

impl<V> Node<V> {
    pub fn get_id(&self) -> u128 {
        self.id
    }
    fn handle_message(&mut self, msg: Message<V>) {
        todo!()
    }

    pub(crate) fn send_message(&self, dst: u128, msg: MessageKind<V>) {
        todo!()
    }

    fn find_node(&mut self, id: u128) {
        todo!()
    }

    fn store(&mut self, key: u128, value: V) {
        todo!()
    }
}
