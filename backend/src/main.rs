#[tokio::main]
async fn main() {
    #[cfg(feature = "process")]
    {
        server::foods::get_foods().await;
    }

    server::start_server().await;
}
