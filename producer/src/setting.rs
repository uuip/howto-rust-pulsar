use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Debug,Deserialize)]
pub struct Setting {
    pub pulsar_addr: String,
    pub topic: String,
    pub sub_name: String,
}

impl Setting {
    pub fn init() -> Self {
        dotenv().ok();
        envy::from_env::<Setting>().unwrap()
    }
}
