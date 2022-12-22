use crate::epoll::Epoll;
use crate::errors::SocketError;

use libc::socket;
//use std::io::prelude::*;
use std::os::fd::FromRawFd; // needed for from_raw_fd
use std::os::fd::RawFd;
use std::os::unix::net::{UnixListener, UnixStream};

pub struct NodeConfig {
    pub name: String,
    pub max_nodes: u32,
}

pub struct Node {}

fn futex_loop() {
    // on tap:
    // check all mapped segments and send data segments
}

fn socket_loop(listener: UnixListener) -> Result<(), SocketError> {
    // state for all connections
    // setup epoll
    let epoll = Epoll::new()?;

    // listen on all known sockets

    // send events:
    // new node (once we receive a futex + descriptor)
    // node death (on break)

    //    loop {
    //            // thread code
    //            // accept connections and process them, spawning a new thread for each one
    //            for stream in listener.incoming() {
    //                match stream {
    //                    Ok(stream) => {
    //                        /* connection succeeded */
    //                        std::thread::spawn(|| handle_client(stream));
    //                    }
    //                    Err(err) => {
    //                        /* connection failed */
    //                        break;
    //                    }
    //                }
    //            }
    //        });
    //
    //    }

    return Ok(());
}

fn make_seq_socket_connection(sock_path: &str) -> Result<UnixStream, SocketError> {
    let fd: RawFd;
    unsafe {
        fd = socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0);
        if fd < 0 {
            return Err(SocketError::new(format!(
                "Failed to construct socket: {}",
                std::io::Error::last_os_error()
            )));
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
        sa_family: libc::AF_UNIX as u16, // 2 bytes
        sa_data: sa_data,                // 14 bytes
    };

    unsafe {
        let ret = libc::connect(fd, &addr, 16);
        if ret < 0 {
            return Err(SocketError::new(format!(
                "Failed to connect to {}: {}",
                sock_path,
                std::io::Error::last_os_error(),
            )));
        }
    }

    unsafe {
        return Ok(UnixStream::from_raw_fd(fd));
    }
}

fn make_seq_socket_listener(sock_path: &str) -> Result<UnixListener, SocketError> {
    let fd: RawFd;
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
        let mut listener: UnixListener;
        let mut self_name: String = Default::default();
        for i in 0..config.max_nodes {
            self_name = format!("\0inps.{:02}", i);
            match make_seq_socket_listener(&self_name) {
                Err(err) => {
                    println!("Could not listen on {}: {}", self_name, err);
                }
                Ok(out) => {
                    listener = out;
                }
            }
        }
        if self_name.len() == 0 {
            return Err(SocketError::new(
                format!("No available sockets").to_string(),
            ));
        }

        // connect to neighbors
        let mut out_connections: Vec<UnixStream> = Default::default();
        for i in 0..config.max_nodes {
            let name = format!("\0inps.{:02}", i);
            if name == self_name {
                continue;
            }

            match make_seq_socket_connection(&name) {
                Err(err) => {
                    println!("Could not connect to {}: {}", name, err);
                }
                Ok(out) => {
                    out_connections.push(out);
                }
            }
        }

        // construct shared memory segment

        //// construct futex
        //let thread = std::thread::spawn(|| {
        //    socket_loop(listener);
        //});

        return Ok(Node {
            //listener: listener,
            //sock_thread: sock_thread,
        });
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
