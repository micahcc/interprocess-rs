use inps::{Node, NodeConfig};
use std::{thread, time};

fn main() {
    let node = Node::new(&NodeConfig {
        name: "hello".to_string(),
        max_nodes: 20,
    })
    .unwrap();

    let cb = |header: &[u8], body: &[u8]| {
        println!("head: {:?}, body: {:?}", header, body);
    };

    node.announce("/ping", "", "", b"");
    node.subscribe("/ping", &cb);

    loop {
        thread::sleep(time::Duration::from_millis(250));
        node.publish("/ping", b"hello_head", b"hello_body");
    }
}
