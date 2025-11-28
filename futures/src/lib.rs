use std::{
    future::Future,
    task::Poll,
    thread,
    time::{Duration, Instant},
};

pub struct AsyncTimer {
    duration: Duration,
    start: Option<Instant>,
}

impl AsyncTimer {
    pub fn new(duration: Duration) -> Self {
        AsyncTimer {
            duration,
            start: Option::None,
        }
    }
}

impl Future for AsyncTimer {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(start) = self.as_ref().start {
            if Instant::now() < (start + self.duration) {
                println!("Timer with {:?} has not yet elapsed!", &self.duration);
                return Poll::Pending;
            }
            println!("Timer elapsed after {:?}", &self.duration);
            return Poll::Ready(());
        }
        let duration = self.duration.clone();
        let waker = cx.waker().clone();
        self.start = Some(Instant::now());
        thread::spawn(move || {
            thread::sleep(duration);
            waker.wake();
        });
        println!("Started a timer with {:?}", &self.duration);
        Poll::Pending
    }
}
