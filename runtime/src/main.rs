use runtime::Executor;

use async_timer::AsyncTimer;

use std::time::Duration;

async fn timering() {
    println!("Starting a 5 second timer...");
    AsyncTimer::new(Duration::from_secs(5)).await;
    println!("5 second timer elapsed!");
}

async fn timering2() {
    println!("Starting a 1 second timer...");
    AsyncTimer::new(Duration::from_secs(1)).await;
    println!("1 second timer elapsed!");
}

async fn looping_timer() {
    for i in 1..10 {
        println!("Starting a {i} second timer...");
        AsyncTimer::new(Duration::from_secs(i)).await;
        println!("{i} second timer elapsed!");
    }
}

fn main() {
    let mut executor = Executor::new();
    executor.schedule(timering());
    executor.schedule(timering2());
    executor.schedule(looping_timer());
    executor.block();
    println!("End of program!");
}
