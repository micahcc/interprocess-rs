use inps::{Node, NodeConfig};
use std::{thread, time};

fn main() {
    let node = Node::new(&NodeConfig {
        name: "hello".to_string(),
        max_nodes: 20,
    })
    .unwrap();

    node.announce("/ping", "", "", b"");

    loop {
        thread::sleep(time::Duration::from_millis(250));
        node.publish("/ping", b"hello_head", b"hello_body");
    }
}
