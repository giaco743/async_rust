// Non-leaf future
async fn process_file() -> std::io::Result<()> {
    let contents = tokio::fs::read("input.txt").await?; // Leaf future
    tokio::fs::write("output.txt", contents.to_ascii_uppercase()).await?; // Leaf future
    Ok(())
}

#[tokio::main]
async fn main() {
    process_file().await.unwrap();
}
