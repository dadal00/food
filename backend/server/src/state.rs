use std::sync::Arc;

use arc_swap::ArcSwap;
use bank::{RemoteBank, get_remote_bank};
use meilisearch_sdk::client::Client;
use redis::aio::ConnectionManager;

use super::{config::Config, database::init_redis, search::init_meilisearch};

pub struct State {
    pub remote_bank: ArcSwap<RemoteBank>,
    pub config: Config,
    pub redis_connection: ConnectionManager,
    pub meili_client: Arc<Client>,
}

impl State {
    pub async fn new() -> Arc<Self> {
        let remote_bank = ArcSwap::from_pointee(get_remote_bank().await.unwrap());

        let config = Config::load();

        let (redis_connection, food_votes) =
            init_redis(&config.redis_url, &remote_bank.load().food_id_to_name).await;

        let meili_client = init_meilisearch(
            &config.meili_url,
            &config.meili_key,
            &remote_bank.load().bank.foods,
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
