use futures::task::noop_waker;
use std::{
    future::Future,
    task::Poll,
    time::{Duration, Instant},
};

pub struct AsyncTimer {
    duration: Duration,
    end: Option<Instant>,
    id: String,
}

impl AsyncTimer {
    pub fn new(duration: Duration, id: &str) -> Self {
        AsyncTimer {
            duration,
            end: Option::None,
            id: id.to_string(),
        }
    }
}

impl Future for AsyncTimer {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(end) = self.as_ref().end {
            if Instant::now() < end {
                println!("Timer {} not yet finished, sleeping some more...", self.id);
                return Poll::Pending;
            }
            println!("Timer {} finished!", self.id);
            return Poll::Ready(());
        }
        self.end = Some(Instant::now() + self.duration);
        println!("Timer {} started for {:?}!", self.id, self.duration);
        Poll::Pending
    }
}

fn main() {
    let mut timers = vec![
        Box::pin(AsyncTimer::new(Duration::from_secs(3), "A")),
        Box::pin(AsyncTimer::new(Duration::from_secs(3), "B")),
    ];

    let waker = noop_waker();
    let mut ctx = std::task::Context::from_waker(&waker);

    while !timers.is_empty() {
        timers.retain_mut(|timer| timer.as_mut().poll(&mut ctx).is_pending());
        std::thread::sleep(Duration::from_millis(200));
    }
}
