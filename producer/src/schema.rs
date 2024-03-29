use pulsar::{producer, DeserializeMessage, Error as PulsarError, Payload, SerializeMessage};
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

impl SerializeMessage for Msg {
    fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
        let payload = serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
        Ok(producer::Message {
            payload,
            ..Default::default()
        })
    }
}

impl DeserializeMessage for Msg {
    type Output = Result<Msg, serde_json::Error>;

    fn deserialize_message(payload: &Payload) -> Self::Output {
        serde_json::from_slice(&payload.data)
    }
}

pub static MSG_SCHEMA: &str = r#"{
  "$id": "https://example.com/test.schema.json",
  "type": "record",
  "name": "Msg",
    "fields": [
      {
        "name": "from_user_id",
        "type": "string"
      },
      {
        "name": "to_user_id",
        "type": "string"
      },
      {
        "name": "coin_code",
        "type": "string"
      },
      {
        "name": "point",
        "type": "float"
      },
      {
        "name": "tag_id",
        "type": "string"
      },
      {
        "name": "store_id",
        "type": "string"
      },
      {
        "name": "gen_time",
        "type": "string"
      },
      {
        "name": "ext_json",
        "type": "string"
      },
      {
        "name": "retry_info",
        "type": [
          "null",
          "string"
        ]
      }
    ]
}"#;
