// TODO: replace keys by any hashable
mod network;
mod node;
use network::Network;

fn main() {
    let mut net = Network::new();
    for _ in 0..3 {
        net.add_node();
    }
    for (k, v) in ('a'..'e').enumerate() {
        net.insert_value(k as u128, String::from(v));
    }

    net.get_value(0);
    net.get_value(1);
    net.kill_node();
}
