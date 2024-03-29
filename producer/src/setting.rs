#[derive(Debug)]
pub struct Setting {
    pub pulsar_addr: String,
    pub topic: String,
    pub sub_name: String,
}

pub fn get_str_env(key: &str) -> String {
    dotenvy::var(key).unwrap_or_else(|_| panic!("lost {key}"))
}

impl Setting {
    pub fn init() -> Self {
        let pulsar_addr = get_str_env("PULSAR_URL");
        let topic = get_str_env("PULSAR_TOPIC");
        let sub_name = get_str_env("PULSAR_SUB_NAME");
        Self {
            pulsar_addr,
            topic,
            sub_name,
        }
    }
}
