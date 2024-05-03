use async_timer::AsyncTimer;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let timer_1 = AsyncTimer::new(Duration::from_secs(2));
    let timer_2 = AsyncTimer::new(Duration::from_secs(4));
    let timer_3 = AsyncTimer::new(Duration::from_secs(6));

    println!("Starting timer with 2 seconds...");
    let handle_1 = tokio::spawn(timer_1);
    println!("Starting timer with 4 seconds...");
    let handle_2 = tokio::spawn(timer_2);
    println!("Starting timer with 6 seconds...");
    let handle_3 = tokio::spawn(timer_3);

    let _ = tokio::join!(
        async {
            handle_1.await.unwrap();
            println!("First timer elapsed after 2 seconds.")
        },
        async {
            handle_2.await.unwrap();
            println!("Second timer elapsed after 2 more seconds.")
        },
        async {
            handle_3.await.unwrap();
            println!("Third timer elapsed after 2 more seconds.")
        },
    );
}
