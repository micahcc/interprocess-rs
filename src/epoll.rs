use crate::errors::SocketError;

pub struct Epoll {
    raw_fd: libc::c_int,
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
            return Ok(Epoll { raw_fd: epollfd });
        }
    }

    fn drop(&mut self) {
        unsafe {
            libc::close(self.raw_fd);
        }
    }
}
