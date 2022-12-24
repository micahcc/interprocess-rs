use crate::epoll::{DescribedInput, Epoll, Packet};
use crate::errors::SocketError;
use crate::event::EventFd;
use crate::futex::Futex;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use libc::socket;
use std::io::prelude::*;
use std::os::fd::AsFd;
use std::os::fd::{AsRawFd, FromRawFd, RawFd}; // needed for from_raw_fd
use std::os::unix::net::{UnixListener, UnixStream};

pub struct NodeConfig {
    pub name: String,
    pub max_nodes: u32,
}

pub struct Node {
    socket_shutdown: EventFd,
    socket_rx: Receiver<Packet>,
    socket_thread_handle: std::thread::JoinHandle<Result<(), SocketError>>,
}

fn futex_loop() {
    // on tap:
    // check all mapped segments and send data segments
}

fn socket_loop(
    listener: UnixListener,
    shutdown: EventFd,
    sender: Sender<Packet>,
) -> Result<(), SocketError> {
    // setup epoll to listen for shutdown and new connections
    let mut epoll = Epoll::new()?;
    epoll.add_listener(listener)?;
    epoll.add_event(0, shutdown)?; // TODO event identifier might be useful for multiple

    // listen on all known sockets
    loop {
        match epoll.next() {
            Ok(DescribedInput::UnixStream(new_stream)) => {
                epoll.add_stream(new_stream)?;
            }
            Ok(DescribedInput::Packet(packet)) => match sender.send(packet) {
                Ok(_) => {}
                Err(err) => {
                    println!(
                        "Failed to enqueue new data, leaving socket loop, err: {}",
                        err
                    );
                    break;
                }
            },
            Ok(DescribedInput::Event(event_id)) => match event_id {
                0 => {
                    break;
                }
                _ => {
                    return Err(SocketError::new(format!(
                        "Received unexpected event id: {}",
                        event_id
                    )));
                }
            },
            Err(err) => {
                // TODO(micah) should descriminate more about the errors
                // TODO(micah) should setup logging
                println!("Error: {}", err);
            }
        }
    }

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
        let mut maybe_listener: Option<UnixListener> = None;
        let mut self_name: String = Default::default();
        for i in 0..config.max_nodes {
            self_name = format!("\0inps.{:02}", i);
            match make_seq_socket_listener(&self_name) {
                Err(err) => {
                    println!("Could not listen on {}: {}", self_name, err);
                }
                Ok(out) => {
                    maybe_listener = Some(out);
                }
            }
        }
        if self_name.len() == 0 {
            return Err(SocketError::new(
                format!("No available sockets").to_string(),
            ));
        }
        let listener = maybe_listener.unwrap();

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
        let futex = Futex::new();

        // construct our futex

        // Construct socket IO thread with
        // - shutdown event fd so we can turn it off
        // - receiver channel for input packets + fds
        // - join handle so we can join when we stop
        let shutdown = EventFd::new()?;
        let dup_shutdown = shutdown.dup()?;
        let (tx, rx): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();
        let socket_thread = std::thread::spawn(|| -> Result<(), SocketError> {
            return socket_loop(listener, dup_shutdown, tx);
        });

        return Ok(Node {
            socket_shutdown: shutdown,
            socket_rx: rx,
            socket_thread_handle: socket_thread,
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
