use std::io::Write;

use chrono::Local;
use env_logger::fmt::style::Color;
use log::{info, Level, LevelFilter};
use once_cell::sync::Lazy;
use pulsar::{producer, proto, Pulsar, TokioExecutor};
use serde_json::json;
use uuid::Uuid;

use crate::setting::Setting;

use crate::schema::{Msg, MSG_SCHEMA};
#[rustfmt::skip]
use crate::model::TokenCode;

mod model;
mod schema;
mod setting;

static SETTING: Lazy<Setting, fn() -> Setting> = Lazy::new(Setting::init);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format(|buf, record| {
            let mut level_style = buf.default_level_style(record.level());
            let reset = level_style.render_reset();
            if record.level() == Level::Warn {
                level_style = level_style.fg_color(Some(Color::Ansi256(206_u8.into())));
            }
            let level_style = level_style.render();
            writeln!(
                buf,
                "{level_style}[{} | line:{:<4}|{}]: {}{reset}",
                Local::now().format("%H:%M:%S"),
                record.line().unwrap_or(0),
                record.level(),
                record.args()
            )
        })
        .init();

    let schema: serde_json::Value = serde_json::from_str(MSG_SCHEMA)?;
    let schema_data = serde_json::to_vec(&schema).unwrap();
    let msg_schema = proto::Schema {
        schema_data,
        r#type: proto::schema::Type::Json as i32,
        ..Default::default()
    };
    let pulsar: Pulsar<TokioExecutor> = Pulsar::builder(&SETTING.pulsar_addr, TokioExecutor)
        .build()
        .await?;
    let mut producer = pulsar
        .producer()
        .with_topic(&SETTING.topic)
        .with_options(producer::ProducerOptions {
            schema: Some(msg_schema),
            ..Default::default()
        })
        .build()
        .await?;

    let uuid = Uuid::new_v4().to_string().replace('-', "");
    let tag_id = format!(
        "{}-{}",
        chrono::Utc::now().date_naive().format("%Y%m%d"),
        uuid
    );
    info!("{tag_id}");

    let message = serde_json::from_value::<Msg>(json!({
        "from_user_id": "200700003",
        "to_user_id": "200700001",
        "order_id": 'a',
        "point": 1,
        "ext_json": "ullamco",
        "coin_code": TokenCode::A,
        "gen_time": 0,
        "tag_id": tag_id,
    }))?;
    producer.send(message).await?.await?;
    producer.close().await.expect("");
    Ok(())
}
