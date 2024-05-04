use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    sync::Mutex,
    task::{Context, Poll, Wake, Waker},
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

type Task = Pin<Box<dyn Future<Output = ()>>>;

struct MyWaker {
    idx: usize,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    thread: thread::Thread,
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        self.ready_queue.lock().unwrap().push(self.idx);
        self.thread.unpark();
    }
}

struct Executor {
    futures: RefCell<HashMap<usize, Task>>,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    next_id: usize,
}

impl Executor {
    fn new() -> Self {
        Executor {
            futures: RefCell::new(HashMap::new()),
            // has to be arc and protected by a mutex, as we want to mutate the ready queue
            // from potentially multiple threads via the waker instances
            ready_queue: Arc::new(Mutex::new(vec![])),
            next_id: 0,
        }
    }

    fn get_waker(&self, idx: usize) -> Arc<MyWaker> {
        Arc::new(MyWaker {
            idx,
            ready_queue: self.ready_queue.clone(),
            thread: thread::current(),
        })
    }

    fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let pinned_future = Box::pin(future);
        self.futures
            .borrow_mut()
            .insert(self.next_id, pinned_future);
        self.ready_queue.lock().unwrap().push(self.next_id);
        self.next_id += 1;
    }
    fn block(&mut self) {
        loop {
            while let Some(idx) = self.ready_queue.lock().unwrap().pop() {
                let mut future = self.futures.borrow_mut().remove(&idx).unwrap();
                let waker: Waker = self.get_waker(idx).into();
                let mut ctx = Context::from_waker(&waker);
                match future.as_mut().poll(&mut ctx) {
                    Poll::Ready(_) => (),
                    Poll::Pending => {
                        self.futures.borrow_mut().insert(idx, future);
                    }
                };
            }
            let tasks_count = self.futures.borrow().len();
            let thread_name = thread::current().name().unwrap_or_default().to_string();
            if tasks_count > 0 {
                println!(
                    "Waiting for tasks to be ready. {} tasks remaining. Parking thread {}.",
                    tasks_count, thread_name,
                );
                thread::park();
            } else {
                println!("Everything done! No tasks left!");
                break;
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
    let mut executor = Executor::new();
    executor.spawn(timering());
    executor.spawn(timering2());
    executor.block();
    println!("End of program!");
}
