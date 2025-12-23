use std::sync::Arc;

use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;

use super::{config::Config, database::init_redis, search::init_meilisearch};

pub struct State {
    pub config: Config,
    pub redis_connection: ConnectionManager,
    pub meili_client: Arc<Client>,
}

impl State {
    pub async fn new() -> Arc<Self> {
        let config = Config::load();

        let redis_future = init_redis(&config.redis_url);
        let meili_future = init_meilisearch(&config.meili_url, &config.meili_key);

        let (redis_connection, meili_client) = tokio::join!(redis_future, meili_future);

        Arc::new(Self {
            config,
            redis_connection,
            meili_client,
        })
    }
}
