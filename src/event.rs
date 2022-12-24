use crate::errors::SocketError;

pub struct EventFd {
    raw_fd: libc::c_int,
}

impl EventFd {
    pub fn new() -> Result<EventFd, SocketError> {
        unsafe {
            let fd = libc::eventfd(0, 0);
            if fd == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct eventfd: {}",
                    std::io::Error::last_os_error()
                )));
            }
            return Ok(EventFd { raw_fd: fd });
        }
    }

    pub fn dup(&self) -> Result<EventFd, SocketError> {
        unsafe {
            let fd = libc::dup(self.raw_fd);
            if fd == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct eventfd: {}",
                    std::io::Error::last_os_error()
                )));
            }
            return Ok(EventFd { raw_fd: fd });
        }
    }

    pub fn incr(&self) -> Result<(), SocketError> {
        unsafe {
            let value: u64 = 1;
            let ptr: *const u64 = &value;
            let ret = libc::write(self.raw_fd, ptr as *const libc::c_void, 8);

            if ret == -1 {
                return Err(SocketError::new(format!(
                    "Failed to decr event: {}",
                    std::io::Error::last_os_error()
                )));
            }

            return Ok(());
        }
    }

    pub fn decr(&self) -> Result<u64, SocketError> {
        unsafe {
            let mut value: u64 = 0;
            let mut ptr: *mut u64 = &mut value;
            let ret = libc::read(self.raw_fd, ptr as *mut libc::c_void, 8);
            if ret == -1 {
                return Err(SocketError::new(format!(
                    "Failed to decr event: {}",
                    std::io::Error::last_os_error()
                )));
            }

            return Ok(value);
        }
    }

    pub fn as_raw_fd(&self) -> std::os::fd::RawFd {
        return self.raw_fd;
    }
}
