use libc::socket;
use std::fmt;

struct ListeningSeqSocket {
    fd: libc::c_int,
}

#[derive(Debug)]
struct SocketError {
    message: String,
}

impl SocketError {
    fn new(descr: String) -> SocketError {
        return SocketError { message: descr };
    }
}

impl ListeningSeqSocket {
    unsafe fn new(sock_path: &str) -> Result<ListeningSeqSocket, SocketError> {
        let fd = socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0);
        if fd < 0 {
            return Err(SocketError::new("Failed to construct socket".to_string()));
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

        {
            let ret = libc::bind(fd, &addr, 16);
            if ret == -1 {
                libc::close(fd);
                return Err(SocketError::new(
                    format!("Failed to bind to {}", ret).to_string(),
                ));
            }
        }

        {
            let ret = libc::listen(fd, 20);
            if ret == -1 {
                libc::close(fd);
                return Err(SocketError::new(
                    format!("Failed to listen to {}", ret).to_string(),
                ));
            }
        }

        return Ok(ListeningSeqSocket { fd: fd });
    }

    unsafe fn poll(&self) -> Result<(), SocketError> {
        let mut from_addr = libc::sockaddr {
            sa_family: libc::AF_UNIX as u16,
            sa_data: [0; 14],
        };
        let mut from_addr_len: u32 = 0;
        let data_socket = libc::accept4(
            self.fd,
            &mut from_addr,
            &mut from_addr_len,
            libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
        );

        if data_socket == -1 {
            return Err(SocketError::new("Failed to accept".to_string()));
        }

        return Ok(());
    }

    unsafe fn drop(&mut self) {
        libc::close(self.fd);
    }
}

fn main() {
    unsafe {
        let conn = ListeningSeqSocket::new("hello").unwrap();
    }
}
