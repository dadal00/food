#[tokio::main]
async fn main() {
    process::load_foods().await;
}
