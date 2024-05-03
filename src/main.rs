use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake},
    thread,
    time::{Duration, Instant},
};

struct AsyncTimer {
    duration: Duration,
    start: Option<Instant>,
}

impl AsyncTimer {
    fn new(duration: Duration) -> Self {
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

static mut READY_QUEUE: Vec<usize> = vec![];

struct Runtime {
    futures: Vec<Pin<Box<dyn Future<Output = ()>>>>,
    ready_queue: &'static mut Vec<usize>,
}

impl Runtime {
    fn new() -> Self {
        unsafe {
            Runtime {
                futures: vec![],
                ready_queue: &mut READY_QUEUE,
            }
        }
    }
}

struct MyWaker {
    idx: usize,
    ready_queue: &'static mut Vec<usize>,
}

impl MyWaker {
    fn new(idx: usize) -> Self {
        unsafe {
            MyWaker {
                idx,
                ready_queue: &mut READY_QUEUE,
            }
        }
    }
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        unsafe { READY_QUEUE.push(self.idx) };
    }
}

impl Runtime {
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let pinned_future = Box::pin(future);
        let idx = self.futures.len();
        self.futures.push(pinned_future);
        self.ready_queue.push(idx);
    }
    fn block(&mut self) {
        loop {
            if let Some(idx) = self.ready_queue.pop() {
                let future = &mut self.futures[idx];
                let waker = Arc::new(MyWaker::new(idx)).into();
                let mut ctx = Context::from_waker(&waker);
                let _ = future.as_mut().poll(&mut ctx);
            }
        }
    }
}

async fn timering() {
    AsyncTimer::new(Duration::from_secs(5)).await;
}

async fn timering2() {
    AsyncTimer::new(Duration::from_secs(1)).await;
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.spawn(timering());
    runtime.spawn(timering2());
    runtime.block();
    println!("Hello, world!");
}
