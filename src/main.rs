use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use chrono::{Local, Utc};
use env_logger::fmt::style::Color;
use ethers::prelude::*;
use futures_util::TryStreamExt;
use log::{debug, error, info, Level, LevelFilter};
use once_cell::sync::{Lazy, OnceCell};
use tokio_postgres::types::ToSql;
use pulsar::{Consumer, Pulsar, SubType, TokioExecutor};
use reqwest::Url;

use crate::action::{persist_one, send_tx};
use crate::error::AppError;
use crate::model::StatusChoice;
use crate::schema::Msg;
use crate::setting::{connection, Setting};

mod action;
mod error;
mod model;
mod schema;
mod setting;

static CHAIN_ID: OnceCell<U256> = OnceCell::new();
static SETTING: Lazy<Setting, fn() -> Setting> = Lazy::new(Setting::init);
const EXC_ST: &str ="update transactions_pool set status_code=$1,status=$2,updated_at=current_timestamp,fail_reason=$3,request_time=$4 where tag_id=$5";

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

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(60))
        .build()?;
    let rpc_url = Url::parse(&SETTING.rpc)?;
    let provider = Http::new_with_client(rpc_url, client);
    let w3 = Arc::new(Provider::new(provider));
    // let w3 = Arc::new(Provider::try_from(rpc_url)?);
    let chain_id = w3.get_chainid().await?;
    CHAIN_ID
        .set(chain_id)
        .unwrap_or_else(|_| panic!("can't set chain_id"));

    let (s, r) = async_channel::bounded::<Msg>((2 * SETTING.batch_size) as usize);

    let pool = connection().await;

    let pulsar: Pulsar<TokioExecutor> = Pulsar::builder(&SETTING.pulsar_addr, TokioExecutor)
        .build()
        .await?;
    let mut consumer: Consumer<Msg, TokioExecutor> = pulsar
        .consumer()
        .with_topic(&SETTING.topic)
        .with_subscription_type(SubType::Shared)
        .with_subscription(&SETTING.sub_name)
        .build()
        .await?;

    let s_pool = pool.clone();
    let s_task = tokio::spawn(async move {
        while let Ok(msg) = consumer.try_next().await {
            if let Some(msg) = msg {
                let data = match msg.deserialize() {
                    Ok(data) => data,
                    Err(e) => {
                        error!("could not deserialize message: {:?}", e);
                        consumer.nack(&msg).await?;
                        continue;
                    }
                };
                let rst = persist_one(&s_pool, &data).await;
                match rst {
                    Ok(_) => {
                        consumer.ack(&msg).await?;
                    }
                    Err(e) => {
                        error!("could not persist message:{e}; {data:?}");
                        if e.to_string()
                            .contains("duplicate key value violates unique constraint")
                        {
                            consumer.ack(&msg).await?;
                        } else {
                            consumer.nack(&msg).await?;
                            continue;
                        }
                    }
                }
                s.send(data).await?;
            }
        }
        Ok::<(), anyhow::Error>(())
    });
    for i in 0..SETTING.batch_size {
        let r = r.clone();
        let pool = pool.clone();
        let w3 = w3.clone();
        tokio::spawn(async move {
            while let Ok(msg) = r.recv().await {
                debug!("{i} running");
                let client = pool
                    .get()
                    .await
                    .unwrap_or_else(|e| panic!("get db connection error: {}", e));
                let request_time = Utc::now();
                let rst = send_tx(&w3, &client, &msg).await;
                match rst {
                    Ok(tx_hash) => {
                        info!("{}:{}", tx_hash.type_name(), tx_hash);
                        let st="update transactions_pool set status_code=202,tx_hash=$1,updated_at=current_timestamp,fail_reason=null,request_time=$2 where tag_id=$3";
                        let _ = client
                            .execute(st, &[&tx_hash, &request_time, &msg.tag_id])
                            .await;
                    }
                    Err(e) => match e {
                        AppError::SQLError(..) | AppError::ProviderError(..) => {
                            error!("sql execution error: {e}");
                            let params: &[&(dyn ToSql + Sync)] = &[
                                &400,
                                &StatusChoice::Fail,
                                &e.to_string(),
                                &request_time,
                                &msg.tag_id,
                            ];
                            let _ = client.execute(EXC_ST, params).await;
                        }
                        _ => {
                            error!("{e}");
                            let params: &[&(dyn ToSql + Sync)] = &[
                                &500,
                                &StatusChoice::Fail,
                                &e.to_string(),
                                &request_time,
                                &msg.tag_id,
                            ];
                            let _ = client.execute(EXC_ST, params).await;
                        }
                    },
                }
            }
        });
    }
    let _ = tokio::join!(s_task);
    Ok(())
}

pub trait AnyExt {
    fn type_name(&self) -> &'static str;
}

impl<T> AnyExt for T {
    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}
