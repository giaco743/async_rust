use std::{
    pin::{Pin, pin},
    task::Poll,
    time::Duration,
};

async fn coroutine() -> usize {
    let mut i = 5;
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("i is {i}");
    i += 3;
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("i is {i}");
    i
}

enum CoroutineState {
    Start,
    Wait1(Pin<Box<dyn Future<Output = ()>>>),
    Wait2(Pin<Box<dyn Future<Output = ()>>>),
    Resolved,
}

struct Coroutine {
    state: CoroutineState,
    i: usize,
}

impl Coroutine {
    fn new() -> Self {
        Coroutine {
            state: CoroutineState::Start,
            i: 0,
        }
    }
}

impl Future for Coroutine {
    type Output = usize;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            match &mut self.state {
                CoroutineState::Start => {
                    self.i = 5;
                    self.state =
                        CoroutineState::Wait1(Box::pin(tokio::time::sleep(Duration::from_secs(2))));
                    continue;
                }
                CoroutineState::Wait1(future) => match pin!(future).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(()) => {
                        println!("i is {}", self.i);
                        self.i += 3;
                        self.state = CoroutineState::Wait2(Box::pin(tokio::time::sleep(
                            Duration::from_secs(2),
                        )));
                        continue;
                    }
                },
                CoroutineState::Wait2(future) => match pin!(future).poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(()) => {
                        self.state = CoroutineState::Resolved;
                        println!("i is {}", self.i);
                        return Poll::Ready(self.i);
                    }
                },
                CoroutineState::Resolved => panic!("Future already resolved"),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let coroutine = Coroutine::new();
    coroutine.await;
}
