use crate::errors::SocketError;
use rand::prelude::*;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};

pub struct SharedSegmentWriter {
    num_messages: u64,
    max_message_bytes: u64,
    raw_fd: RawFd,
    next_seq: u64,
}

pub struct SharedSegmentReader {
    num_messages: u64,
    max_message_bytes: u64,
    raw_fd: RawFd,
    next_seq: u64,
}

#[repr(C)]
struct MessageMeta {
    // negative sequences are in flight, not to be touched
    // zeros are unoccupied
    // positive sequences are previously written
    seq: i64,
    crc: u64,
    offset: u64,
}

fn headsize(num_messages: usize) -> usize {
    // top:
    // u64 segment ID
    // each message:
    // u64 seq
    // u64 crc
    // u64 offset
    return 8 + 3 * 8 * num_messages;
}

impl SharedSegmentWriter {
    // TODO(micah) create allocate function to eliminate a copy
    //pub fn allocate(n_bytes: usize) -> {
    //}

    pub fn write(&mut self, arr: &[u8]) {
        // choose the lowest sequence to overwrite

        //
    }

    pub fn new(num_messages: usize, max_bytes: usize) -> Result<SharedSegmentWriter, SocketError> {
        unsafe {
            // TODO(micah) if we want to support GPU memory we should split head
            // and body so head can travel over the CPU

            let fd = libc::memfd_create("segment\0".as_ptr() as *const i8, 0);
            if fd == -1 {
                return Err(SocketError::new(format!(
                    "Failed to construct memfd: {}",
                    std::io::Error::last_os_error()
                )));
            }

            // NOTE: ftruncate fills the file with zeros, zero sequence means unused
            // so we don't need to initialize the header other than to fill the
            // id field
            let body_start = headsize(num_messages);
            let n_bytes = max_bytes + headsize(num_messages);
            {
                let ret = libc::ftruncate(fd, n_bytes as i64);
                if ret == -1 {
                    libc::close(fd);
                    return Err(SocketError::new(format!(
                        "Failed to truncate mem to 4 bytes: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }

            // initialize header
            let mut rng = rand::thread_rng();
            let id: u64 = rng.gen();
            {
                let ptr: *const u64 = &id;
                let ret = libc::pwrite(fd, ptr as *const libc::c_void, 8, 0);
                if ret == -1 {
                    libc::close(fd);
                    return Err(SocketError::new(format!(
                        "Failed to write unique ID: {}",
                        std::io::Error::last_os_error()
                    )));
                }
            }

            return Ok(Self {
                max_message_bytes: max_bytes as u64,
                num_messages: num_messages as u64,
                raw_fd: fd,
                next_seq: 1,
            });
        }
    }
}
