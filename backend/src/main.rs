#[tokio::main]
async fn main() {
    #[cfg(feature = "process")]
    {
        food::foods::get_foods().await;
    }

    food::start_server().await;
}
