#[tokio::main]
async fn main() {
    if let Err(e) = mural_client::run().await {
        eprintln!("{}", e);
    }
}
