use pulsar::{DeserializeMessage, Payload};
use serde::{Deserialize, Serialize};

use crate::model::TokenCode;

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg {
    pub from_user_id: String,
    pub to_user_id: String,
    pub order_id: String,
    pub point: i64,
    pub coin_code: TokenCode,
    pub gen_time: i64,
    pub tag_id: String,
    pub ext_json: Option<String>,
    pub store_id: Option<String>,
}

impl DeserializeMessage for Msg {
    type Output = Result<Msg, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}
