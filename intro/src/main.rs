// Non-leaf future
async fn process_file() -> std::io::Result<()> {
    println!("Reading file...");
    let contents = tokio::fs::read("input.txt").await?; // Leaf future

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("Processing file...");
    tokio::fs::write("output.txt", contents.to_ascii_uppercase()).await?; // Leaf future

    println!("...file operations done!");
    Ok(())
}

async fn another_async_fn() {
    println!("Doing something asynchronously...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("...do more...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("Done!");
}

// Desugaring

#[tokio::main]
async fn main() {
    // asynchronous but NOT concurrent
    process_file().await.unwrap();
    another_async_fn().await;

    // Truly concurrent execution using tokio::join!
    // let (res_a, _) = tokio::join!(process_file(), another_async_fn());

    // res_a.unwrap();
    // println!("All tasks completed.");
}
