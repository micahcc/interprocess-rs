use libc::socket;
use std::fmt;

#[derive(Debug)]
struct SocketError {
    message: String,
}

impl SocketError {
    fn new(descr: String) -> SocketError {
        return SocketError { message: descr };
    }
}

struct ConnectingSeqSocket {
    fd: libc::c_int,
}

impl ConnectingSeqSocket {
    unsafe fn new(sock_path: &str) -> Result<ConnectingSeqSocket, SocketError> {
        let fd = socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0);
        if fd < 0 {
            return Err(SocketError::new("Failed to construct socket".to_string()));
        }

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

        let ret = libc::connect(fd, &addr, 16);
        if ret < 0 {
            libc::close(fd);
            return Err(SocketError::new(
                format!("Failed to connect to {}", ret).to_string(),
            ));
        }

        return Ok(ConnectingSeqSocket { fd: fd });
    }

    unsafe fn drop(&mut self) {
        libc::close(self.fd);
    }
}

fn main() {
    unsafe {
        let conn = ConnectingSeqSocket::new("hello").unwrap();
    }
}
