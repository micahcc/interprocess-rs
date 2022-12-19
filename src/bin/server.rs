//use libc::socket;
//use std::fmt;
//
//struct ListeningSeqSocket {
//    fd: libc::c_int,
//}
//
//impl ListeningSeqSocket {
//    unsafe fn new(sock_path: &str) -> Result<ListeningSeqSocket, SocketError> {
//        return Ok(ListeningSeqSocket { fd: fd });
//    }
//
//    unsafe fn poll(&self) -> Result<(), SocketError> {
//        let mut from_addr = libc::sockaddr {
//            sa_family: libc::AF_UNIX as u16,
//            sa_data: [0; 14],
//        };
//        let mut from_addr_len: u32 = 0;
//        let data_socket = libc::accept4(
//            self.fd,
//            &mut from_addr,
//            &mut from_addr_len,
//            libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
//        );
//
//        if data_socket == -1 {
//            return Err(SocketError::new("Failed to accept".to_string()));
//        }
//
//        return Ok(());
//    }
//
//    unsafe fn drop(&mut self) {
//        libc::close(self.fd);
//    }
//}

fn main() {
    //unsafe {
    //    let conn = ListeningSeqSocket::new("hello").unwrap();
    //}
}
