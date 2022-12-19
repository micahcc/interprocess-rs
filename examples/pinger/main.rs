use inps::{Node, NodeConfig};

fn main() {
    let node = Node::new(NodeConfig {
        name: "hello".to_string(),
        max_nodes: 20,
    });
}
