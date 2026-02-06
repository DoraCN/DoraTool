#[tokio::main]
async fn main() {
    if let Err(err) = doratool::run().await {
        println!("Server Err: {}", err);
    }
}
