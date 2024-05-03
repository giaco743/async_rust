use std::{
    fmt::Write,
    pin::{Pin, pin},
    task::Poll,
    time::Duration,
};

use async_timer::AsyncTimer;
use runtime::Executor;

async fn coroutine_b() {
    let mut buffer = String::new();
    let writer = &mut buffer;
    println!("Coroutine started. Buffer: {}", writer);
    AsyncTimer::new(Duration::from_secs(2)).await;
    write!(writer, "Hello ").unwrap();
    println!("2 seconds elapsed. Buffer: {}", writer);
    AsyncTimer::new(Duration::from_secs(2)).await;
    write!(writer, "World!!!").unwrap();
    println!("4 seconds elapsed. Buffer: {}", writer);
}

enum CoroutineState {
    Start,
    Wait1(Pin<Box<dyn Future<Output = ()>>>),
    Wait2(Pin<Box<dyn Future<Output = ()>>>),
    Resolved,
}

#[derive(Default)]
struct Stack {
    buffer: Option<String>,
    writer: Option<*mut String>,
}

struct CoroutineB {
    stack: Stack,
    state: CoroutineState,
}

unsafe impl Send for CoroutineB {}

impl CoroutineB {
    fn new() -> Self {
        CoroutineB {
            stack: Stack::default(),
            state: CoroutineState::Start,
        }
    }
}

impl Future for CoroutineB {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            match &mut self.state {
                CoroutineState::Start => {
                    self.stack.buffer = Some(String::new());
                    self.stack.writer = Some(self.stack.buffer.as_mut().unwrap());
                    let writer = unsafe { &mut *self.stack.writer.take().unwrap() };
                    println!("Coroutine started. Buffer: {}", *writer);
                    self.state =
                        CoroutineState::Wait1(Box::pin(AsyncTimer::new(Duration::from_secs(2))));
                    self.stack.writer = Some(writer);
                    continue;
                }
                CoroutineState::Wait1(future) => match pin!(future).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(()) => {
                        let writer = unsafe { &mut *self.stack.writer.take().unwrap() };
                        write!(writer, "Hello ").unwrap();
                        println!("2 seconds elapsed. Buffer: {}", writer);
                        self.state = CoroutineState::Wait2(Box::pin(AsyncTimer::new(
                            Duration::from_secs(2),
                        )));
                        self.stack.writer = Some(writer);
                        continue;
                    }
                },
                CoroutineState::Wait2(future) => match pin!(future).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(()) => {
                        let writer = unsafe { &mut *self.stack.writer.take().unwrap() };
                        write!(writer, "World!!!").unwrap();
                        println!("4 seconds elapsed. Buffer: {}", writer);
                        self.state = CoroutineState::Resolved;
                        // free resources
                        let _ = self.stack.buffer.take();
                        return Poll::Ready(());
                    }
                },
                CoroutineState::Resolved => panic!("Future already resolved"),
            }
        }
    }
}

fn main() {
    let mut future = CoroutineB::new();
    let future_b = CoroutineB::new();
    future = future_b;
    let handle = std::thread::spawn(move || {
        let mut executor = Executor::new();
        executor.schedule(future);
        executor.block();
    });
    handle.join().unwrap();
    println!("End of program!");
}
