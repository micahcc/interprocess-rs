use crate::errors::SocketError;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};

pub struct Futex {
    raw_fd: RawFd,
}

impl Futex {
    pub fn new() -> Result<Futex, SocketError> {
        unsafe {
            let fd = libc::memfd_create("futex\0".as_ptr() as *const i8, 0);
            if fd == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct memfd: {}",
                    std::io::Error::last_os_error()
                )));
            }

            let ret = libc::ftruncate(fd, 4);
            if ret == -1 {
                libc::close(fd);
                return Err(SocketError::new(format!(
                    "Failed to truncate mem to 4 bytes: {}",
                    std::io::Error::last_os_error()
                )));
            }

            return Ok(Self { raw_fd: fd });
        }
    }
}
