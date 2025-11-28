#[tokio::main]
async fn main() {
    let timer_1 = AsyncTimer::new(Duration::from_secs(2));
    let timer_2 = AsyncTimer::new(Duration::from_secs(4));
    let timer_3 = AsyncTimer::new(Duration::from_secs(6));

    let handle_1 = tokio::spawn(timer_1);
    let handle_2 = tokio::spawn(timer_2);
    let handle_3 = tokio::spawn(timer_3);

    let _ = tokio::join!(handle_1, handle_2, handle_3);
}
