use inps::{Node, NodeConfig};

fn main() {
    let node = Node::new(NodeConfig {
        name: "hello",
        max_nodes: 20,
    });
}
