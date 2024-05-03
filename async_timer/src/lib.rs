use std::{future::Future, task::Poll, thread, time::Duration};

pub struct AsyncTimer {
    duration: Duration,
    started: bool,
}

impl AsyncTimer {
    pub fn new(duration: Duration) -> Self {
        AsyncTimer {
            duration,
            started: false,
        }
    }
}

impl Future for AsyncTimer {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if !self.started {
            self.started = true;

            let duration = self.duration.clone();
            let waker = cx.waker().clone();
            // In a real async runtime, you wouldn't spawn a thread like this,
            // but use syscalls instead to make use of timers and events provided by the OS.
            thread::spawn(move || {
                thread::sleep(duration);
                println!(
                    "Timer expired! Calling waker.wake() \
                    to tell the runtime that the future is ready to be polled again..."
                );
                waker.wake();
            });
            return Poll::Pending;
        }
        Poll::Ready(())
    }
}
