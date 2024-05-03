use std::{
    pin::{Pin, pin},
    task::Poll,
    time::Duration,
};

async fn coroutine() {
    println!("Coroutine started");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("2 seconds elapsed");
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("4 seconds elapsed");
}

fn coroutine_transformed() -> impl std::future::Future<Output = ()> {
    enum CoroutineState {
        Start,
        Wait1(Pin<Box<dyn Future<Output = ()>>>),
        Wait2(Pin<Box<dyn Future<Output = ()>>>),
        Resolved,
    }

    struct Coroutine {
        state: CoroutineState,
    }

    impl Future for Coroutine {
        type Output = ();

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            loop {
                match &mut self.state {
                    CoroutineState::Start => {
                        println!("Coroutine started");
                        self.state = CoroutineState::Wait1(Box::pin(tokio::time::sleep(
                            Duration::from_secs(2),
                        )));
                        continue;
                    }
                    CoroutineState::Wait1(future) => match pin!(future).poll(cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(()) => {
                            println!("2 seconds elapsed");
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
                            println!("4 seconds elapsed");
                            return Poll::Ready(());
                        }
                    },
                    CoroutineState::Resolved => panic!("Future already resolved"),
                }
            }
        }
    }

    Coroutine{
        state: CoroutineState::Start
    }
}

#[tokio::main]
async fn main() {
    let coroutine = coroutine_transformed();
    coroutine.await;
}
