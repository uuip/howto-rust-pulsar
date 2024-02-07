use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use duplicate::duplicate_item;
use postgres_from_row::FromRow;
use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TokenCode {
    A,
    B,
    C,
    D,
    E,
}

#[derive(Clone, Debug, Serialize, Deserialize, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum StatusChoice {
    Pending,
    Success,
    Fail,
    // #[strum(serialize = "timeout")]
    Timeout,
}

#[duplicate_item(type_name;[TokenCode];[StatusChoice])]
impl FromSql<'_> for type_name {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Self::from_str(std::str::from_utf8(raw)?).map_err(|e| e.into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

#[duplicate_item(type_name;[TokenCode];[StatusChoice])]
impl ToSql for type_name {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized + Display,
    {
        format!("{}", self).to_sql(ty, out)
    }

    fn accepts(_ty: &Type) -> bool
    where
        Self: Sized,
    {
        true
    }

    to_sql_checked!();
}

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct Transaction {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub success_time: Option<DateTime<Utc>>,
    pub request_time: Option<DateTime<Utc>>,
    pub status: StatusChoice,
    pub status_code: i32,
    pub block_number: Option<i64>,
    pub fail_reason: Option<String>,
    pub nonce: Option<i64>,
    pub gas: Option<i64>,
    pub tx_hash: Option<String>,
    pub from_user_id: String,
    pub to_user_id: String,
    pub point: f64,
    pub tag_id: String,
    pub coin_code: String,
    pub ext_json: String,
    pub gen_time: String,
    pub store_id: Option<String>,
}
