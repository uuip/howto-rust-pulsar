use std::str::FromStr;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

#[derive(Debug)]
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

pub fn get_str_env(key: &str) -> String {
    dotenvy::var(key).unwrap_or_else(|_| panic!("lost {key}"))
}

impl Setting {
    pub fn init() -> Self {
        let pulsar_addr = get_str_env("PULSAR_URL");
        let topic = get_str_env("PULSAR_TOPIC");
        let sub_name = get_str_env("PULSAR_SUB_NAME");
        let rpc = get_str_env("RPC");
        let batch_size = dotenvy::var("BATCH_SIZE")
            .expect("lost BATCH_SIZE")
            .parse::<i32>()
            .unwrap_or(40);
        let token_a = get_str_env("TOKEN_A");
        let token_b = get_str_env("TOKEN_B");
        let token_c = get_str_env("TOKEN_C");
        let token_d = get_str_env("TOKEN_D");
        let token_e = get_str_env("TOKEN_E");
        Self {
            pulsar_addr,
            topic,
            sub_name,
            rpc,
            batch_size,
            token_a,
            token_b,
            token_c,
            token_d,
            token_e,
        }
    }
}

pub async fn connection() -> Pool {
    let db_url = dotenvy::var("DB_URL").unwrap_or_else(|_| panic!("lost DB_URL"));
    let mut pg_config =tokio_postgres::Config::from_str(&db_url).unwrap();
    pg_config.options("-c LC_MESSAGES=en_US.UTF-8");
    let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast
        };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    Pool::builder(mgr).max_size(100).build().unwrap()
}
