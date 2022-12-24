use crate::errors::SocketError;
use crate::event::EventFd;
use sendfd::RecvWithFd;
use std::collections::HashMap;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream}; // needed for from_raw_fd

enum Described {
    UnixListener(UnixListener),
    UnixStream(UnixStream),
    EventFd((EventFd, u64)),
}

pub struct Packet {
    bytes: Vec<u8>,
    fds: Vec<RawFd>,
}

pub enum DescribedInput {
    UnixStream(UnixStream), // product of listener
    Packet(Packet),         // produce of unix stream
    Event(u64),             // product of event
}

pub struct Epoll {
    raw_fd: libc::c_int,
    described: HashMap<u64, Described>,
}

fn get_stream_input(stream: &UnixStream) -> Result<DescribedInput, SocketError> {
    let mut bytes: Vec<u8> = vec![0; 1024];
    let mut fds: Vec<RawFd> = vec![-1; 3];

    match stream.recv_with_fd(&mut bytes, &mut fds) {
        Err(err) => {
            return Err(SocketError::new(format!(
                "Failed to receive from stream: {}",
                err
            )));
        }
        Ok((nbytes, nfds)) => {
            println!("received: {}B, {}fd", nbytes, nfds);
            bytes.truncate(nbytes);
            fds.truncate(nfds);
            return Ok(DescribedInput::Packet(Packet {
                bytes: bytes,
                fds: fds,
            }));
        }
    }
}

fn new_connection(listener: &UnixListener) -> Result<DescribedInput, SocketError> {
    match listener.incoming().next() {
        Some(Ok(stream)) => {
            return Ok(DescribedInput::UnixStream(stream));
        }
        Some(Err(err)) => {
            return Err(SocketError::new(format!(
                "Failed to open connection: {}",
                err
            )));
        }
        None => {
            return Err(SocketError::new(
                "Expected a new connection but didn't get one".to_string(),
            ));
        }
    }
}

impl Epoll {
    pub fn new() -> Result<Epoll, SocketError> {
        unsafe {
            let epollfd = libc::epoll_create1(0);
            if epollfd == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct epoll: {}",
                    std::io::Error::last_os_error()
                )));
            }
            return Ok(Epoll {
                raw_fd: epollfd,
                described: Default::default(),
            });
        }
    }

    fn add_trigger(&mut self, fd: std::os::fd::RawFd) -> Result<(), SocketError> {
        let mut event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: fd as u64,
        };
        unsafe {
            let ret = libc::epoll_ctl(self.raw_fd, libc::EPOLL_CTL_ADD, fd, &mut event);
            if ret == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct epoll: {}",
                    std::io::Error::last_os_error()
                )));
            }
        }
        return Ok(());
    }

    pub fn add_stream(&mut self, stream: UnixStream) -> Result<(), SocketError> {
        let key: u64 = stream.as_raw_fd() as u64;
        self.add_trigger(stream.as_raw_fd())?;
        self.described.insert(key, Described::UnixStream(stream));
        return Ok(());
    }

    pub fn add_event(&mut self, id: u64, event: EventFd) -> Result<(), SocketError> {
        let key: u64 = event.as_raw_fd() as u64;
        self.add_trigger(event.as_raw_fd())?;
        self.described.insert(key, Described::EventFd((event, id)));
        return Ok(());
    }

    pub fn add_listener(&mut self, listener: UnixListener) -> Result<(), SocketError> {
        let key: u64 = listener.as_raw_fd() as u64;
        self.add_trigger(listener.as_raw_fd())?;
        self.described
            .insert(key, Described::UnixListener(listener));
        return Ok(());
    }

    pub fn next(&mut self) -> Result<DescribedInput, SocketError> {
        unsafe {
            let mut event = libc::epoll_event { events: 0, u64: 0 };
            let ret = libc::epoll_wait(self.raw_fd, &mut event, 1, -1);
            if ret < 0 {
                return Err(SocketError::new(format!(
                    "Failed to poll: {}",
                    std::io::Error::last_os_error(),
                )));
            }

            let key: u64 = event.u64;
            match self.described.get(&key) {
                Some(Described::UnixStream(stream)) => {
                    return get_stream_input(stream);
                }
                Some(Described::UnixListener(listener)) => {
                    return new_connection(listener);
                }
                Some(Described::EventFd((event, id))) => {
                    // decrement the event and return the id for the user to
                    // match up with
                    event.decr()?;
                    return Ok(DescribedInput::Event(*id));
                }
                None => {
                    return Err(SocketError::new(format!("Missing key: {}", key)));
                }
            }
        }
    }

    fn drop(&mut self) {
        unsafe {
            libc::close(self.raw_fd);
        }
    }
}
