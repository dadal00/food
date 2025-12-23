use std::sync::Arc;

use bank::{foods::Bank, get_bank_remote};
use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;

use super::{config::Config, database::init_redis, search::init_meilisearch};

pub struct State {
    pub bank: Bank,
    pub config: Config,
    pub redis_connection: ConnectionManager,
    pub meili_client: Arc<Client>,
}

impl State {
    pub async fn new() -> Arc<Self> {
        let bank = get_bank_remote().await;

        let config = Config::load();

        let (redis_connection, food_votes) = init_redis(&config.redis_url, &bank).await;
        let meili_client = init_meilisearch(&config.meili_url, &config.meili_key, food_votes).await;

        Arc::new(Self {
            bank,
            config,
            redis_connection,
            meili_client,
        })
    }
}
