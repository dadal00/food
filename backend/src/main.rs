#[tokio::main]
async fn main() {
    food::start_server().await;
}
