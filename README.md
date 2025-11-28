# Async Rust

## Futures

A future is a representation of some operation that will be completed in the future.
1) **Poll phase:** A future is polled, which results in the task progressing until it is finished and returns `Poll::Ready<T>` or until it reaches a point where it can no longer make progress and returns `Poll::Pending` .
2) **Wait phase:** The future is registered at the reactor and waits for the event to happen. Control is given back to the executor which can poll other futures that are ready to make progress.
3) **Wake phase:** The reactor notifies the executor via a waker that the event happened and the future is ready to be polled again.

### Non-Leaf Futures

Non leaf futures are the type of futures users of a runtime write. They are pausable/resumable functions that start with the `async` keyword and `await` other futures.
When awaiting a non-leaf future the future is polled and progresses until it reaches a leaf future returning `Poll::Pending` .

### Leaf Futures

Leaf futures are usually provided by the runtime and perform some I/O operation in a non-blocking way. They do so by leveraging operating system and hardware support such as interrupts.
The leaf futures are usually called by non-leaf futures and are the actually await points.

```rust
// Non-leaf future
async fn process_file() -> std::io::Result<()> {
    let contents = tokio::fs::read("input.txt").await?; // Leaf future
    tokio::fs::write("output.txt", contents.to_ascii_uppercase()).await?; // Leaf future
    Ok(())
}
```

## Non-Leaf futures aka async functions

The compiler transforms async functions...

```rust
async fn coroutine() -> usize {
    let mut i = 5;
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("i is {i}");
    i += 3;
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("i is {i}");
    i
}
```

...into state machines:

```rust
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

```

## The Runtime

Runtimes can be divided into two parts a **Reactor**, which reacts on and dispatches OS events to the **Executor**, which schedules and polls the futures.
The Reactor notifies the Executor that a future is ready to be polled to progress to its next state via a **Waker**.
In this chapter we want to mainly focus on the **Executor** to drive futures to completion, when they are ready to be polled again.

### The Executor

The minimal Executor we want to implement allows us to schedule a number of async tasks and then call block on it, to drive them concurrently to completion.
To keep track of all the scheduled tasks we assign them a unique ID and place them in a HashMap. The `ready_queue` keeps track of all the tasks, that are ready to be polled again via their IDs.
As we want to have `poll` called at least once on all the scheduled tasks, when calling `block` on the Executor, we also have to put the ID into the `ready_queue` when scheduling a task.

When calling `block` on the Executor the scheduled futures are polled once one by one handing out a waker for each of them so the Executor can be notified when they are ready to be polled again (usually the Reactor would take care of this in an efficient manner).

The ID of the next task to be polled is obtained by popping the first entry in the `ready_queue` and removing the task from the `futures` map. The task is then polled by handing out a waker.
The waker has a reference to the ready queue and the ID of the task that was polled with this waker, so that it can put the task back onto the `ready_queue` when the event the task is waiting for has occured. It also has the thread ID of the thread we are running on, to wake up the executor in case it has no tasks ready to poll and thus has given control back to the OS, to do other stuff.
If the result is `Poll::Ready` we just go on with the next task, if the result is `Poll::Pending` we put it into the `tasks` map again before continuing with the next one in order to poll it again in the future.

When the `ready_queue` is empty and there are no tasks left to poll, we hand over control to the OS, to do other stuff, by calling `thread::park()` the thread will be unparked when wake is called on a Waker.
