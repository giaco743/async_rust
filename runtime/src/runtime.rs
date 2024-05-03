use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    thread,
};

type Task = Pin<Box<dyn Future<Output = ()>>>;

pub struct MyWaker {
    task_id: usize,
    ready_queue: Arc<Mutex<VecDeque<usize>>>,
    thread: thread::Thread,
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        self.ready_queue.lock().unwrap().push_back(self.task_id);
        self.thread.unpark();
    }
}

pub struct Executor {
    tasks: HashMap<usize, Task>,
    ready_queue: Arc<Mutex<VecDeque<usize>>>,
    next_id: usize,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: HashMap::new(),
            ready_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: 0,
        }
    }

    pub fn schedule(&mut self, future: impl Future<Output = ()> + 'static) {
        let pinned_future = Box::pin(future);
        self.tasks.insert(self.next_id, pinned_future);
        self.ready_queue.lock().unwrap().push_back(self.next_id);
        self.next_id += 1;
    }

    pub fn block(&mut self) {
        loop {
            while let Some(id) = self.ready_queue.lock().unwrap().pop_front() {
                let mut future = self.tasks.remove(&id).unwrap();
                let waker: Waker = self.from_id(id).into();
                let mut ctx = Context::from_waker(&waker);
                match future.as_mut().poll(&mut ctx) {
                    Poll::Ready(_) => (),
                    Poll::Pending => {
                        self.tasks.insert(id, future);
                    }
                };
            }
            let tasks_count = self.tasks.len();
            let thread_name = thread::current().name().unwrap_or_default().to_string();
            if tasks_count > 0 {
                println!(
                    "⏸️ Waiting for tasks to be ready. {tasks_count} tasks remaining. Parking thread {thread_name}.",
                );
                // We block aka give control back to the OS, as there are no more tasks to poll,
                // the OS can do other stuff in the meantime.
                thread::park();
                println!("▶️ Thread {thread_name} unparked. Continuing with ready tasks...",);
            } else {
                println!("⏹️ Everything done! No tasks left!");
                break;
            }
        }
    }

    fn from_id(&self, id: usize) -> Arc<MyWaker> {
        Arc::new(MyWaker {
            task_id: id,
            ready_queue: self.ready_queue.clone(),
            thread: thread::current(),
        })
    }
}
