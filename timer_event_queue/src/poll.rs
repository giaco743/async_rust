use crate::ffi;
use std::{
    io::{self, Result},
    net::TcpStream,
    os::fd::AsRawFd,
};
type Events = Vec<ffi::Event>;
pub struct Poll {
    registry: Registry,
}
impl Poll {
    /// Create a new event queue
    pub fn new() -> Result<Self> {
        let res = unsafe { ffi::epoll_create(1) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            registry: Registry { raw_fd: res },
        })
    }
    /// Register interest for event notifications using the registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    /// Block the thread until an event is ready or it times out
    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let fd = self.registry.raw_fd;
        let timeout = timeout.unwrap_or(-1);
        let max_events = events.capacity() as i32;
        let res = unsafe { ffi::epoll_wait(fd, events.as_mut_ptr(), max_events, timeout) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        unsafe { events.set_len(res as usize) };
        Ok(())
    }
}
pub struct Registry {
    raw_fd: i32,
}
impl Registry {
    /// Register interest for an event notification
    /// (for simplicity only Tcp events are considered,
    ///  but we could extend this with an abstraction over
    ///  different event sources)
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        let mut event = ffi::Event {
            events: interests as u32,
            epoll_data: token,
        };
        let op = ffi::EPOLL_CTL_ADD;
        let res = unsafe { ffi::epoll_ctl(self.raw_fd, op, source.as_raw_fd(), &mut event) };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe { ffi::close(self.raw_fd) };
        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("ERROR: {err:?}");
        }
    }
}
