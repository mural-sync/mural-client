#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(e) = mural_client::run().await {
        eprintln!("{}", e);
    }
}
