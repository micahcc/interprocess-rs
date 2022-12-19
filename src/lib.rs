use libc::socket;
use std::fmt;
use std::io::prelude::*;
use std::os::fd::RawFd;
//use std::os::unix::net::listener::FromRawFd;
use std::os::fd::FromRawFd;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct NodeConfig {
    pub name: String,
    pub max_nodes: u32,
}

pub struct Node {}

#[derive(Debug)]
pub struct SocketError {
    message: String,
}

impl SocketError {
    pub fn new(descr: String) -> SocketError {
        return SocketError { message: descr };
    }
}

fn make_seq_socket_raw(sock_path: &str) -> Result<UnixListener, SocketError> {
    let mut fd: RawFd = -1;
    unsafe {
        fd = socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0);
        if fd < 0 {
            return Err(SocketError::new("Failed to construct socket".to_string()));
        }
    }

    //name.sun_family = AF_UNIX;
    // strncpy(name.sun_path, SOCKET_NAME, sizeof(name.sun_path) - 1);
    // need 0 at begin, and 0 at end, so only have 12 characters
    if sock_path.len() > 12 {
        return Err(SocketError::new(format!(
            "Socket name should be < 12 characters (got {})",
            sock_path.len()
        )));
    }

    let mut sa_data: [libc::c_char; 14] = [0; 14];
    for (i, c) in sock_path.bytes().enumerate() {
        sa_data[i + 1] = c as i8;
    }
    let addr = libc::sockaddr {
        sa_family: libc::AF_UNIX as u16,
        sa_data: sa_data,
    };

    unsafe {
        let ret = libc::bind(fd, &addr, 16);
        if ret == -1 {
            libc::close(fd);
            return Err(SocketError::new(
                format!("Failed to bind to {}", ret).to_string(),
            ));
        }
    }

    unsafe {
        let ret = libc::listen(fd, 20);
        if ret == -1 {
            libc::close(fd);
            return Err(SocketError::new(
                format!("Failed to listen to {}", ret).to_string(),
            ));
        }
    }

    unsafe {
        return Ok(UnixListener::from_raw_fd(fd));
    }
}

impl Node {
    pub fn new(config: &NodeConfig) -> Result<Node, SocketError> {
        // construct socket
        for i in 0..config.max_nodes {
            let name = format!("\0inps.{:02}", i);
            let listener = make_seq_socket_raw("\0test")?;
        }

        // construct shared memory segment

        // construct futex

        return Ok(Node {});
    }

    pub fn announce(
        &self,
        topic: &str,
        head_type_name: &str,
        body_type_name: &str,
        proto_defs: &[u8],
    ) {
    }

    pub fn publish(&self, topic: &str, head: &[u8], body: &[u8]) {}
    pub fn subscribe(&self, topic: &str, cb: &dyn Fn(&[u8], &[u8])) {}
}
