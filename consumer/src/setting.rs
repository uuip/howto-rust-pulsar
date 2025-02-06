use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use std::str::FromStr;
use dotenvy::dotenv;
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Debug, Deserialize)]
pub struct Setting {
    pub pulsar_addr: String,
    pub topic: String,
    pub sub_name: String,
    pub rpc: String,
    pub batch_size: i32,
    pub token_a: String,
    pub token_b: String,
    pub token_c: String,
    pub token_d: String,
    pub token_e: String,
}

impl Setting {
    pub fn init() -> Self {
        dotenv().ok();
        envy::from_env::<Setting>().unwrap()
    }
}


pub async fn connection() -> Pool {
    let db_url = dotenvy::var("DB_URL").unwrap_or_else(|_| panic!("lost DB_URL"));
    let mut pg_config = tokio_postgres::Config::from_str(&db_url).unwrap();
    pg_config.options("-c LC_MESSAGES=en_US.UTF-8");
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    Pool::builder(mgr).max_size(100).build().unwrap()
}
