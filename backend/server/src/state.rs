use std::sync::Arc;

use bank::{RemoteBank, get_remote_bank};
use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;

use super::{config::Config, database::init_redis, search::init_meilisearch};

pub struct State {
    pub remote_bank: RemoteBank,
    pub config: Config,
    pub redis_connection: ConnectionManager,
    pub meili_client: Arc<Client>,
}

impl State {
    pub async fn new() -> Arc<Self> {
        let remote_bank = get_remote_bank().await;

        let config = Config::load();

        let (redis_connection, food_votes) =
            init_redis(&config.redis_url, &remote_bank.food_id_to_name).await;

        let meili_client = init_meilisearch(
            &config.meili_url,
            &config.meili_key,
            &remote_bank.bank.foods,
            &food_votes,
        )
        .await;

        Arc::new(Self {
            remote_bank,
            config,
            redis_connection,
            meili_client,
        })
    }
}
