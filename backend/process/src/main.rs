#[tokio::main]
async fn main() {
    process::fetch_foods().await;
}
