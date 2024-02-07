use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

use deadpool_postgres::Pool;
use ethers::abi::AbiEncode;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::prelude::*;
use log::debug;
use once_cell::sync::Lazy;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::schema::Msg;
use crate::{CHAIN_ID, SETTING};

abigen!(Erc20Token, "erc20_abi.json");

static FIELDS: Lazy<String> = Lazy::new(|| {
    [
        "from_user_id",
        "to_user_id",
        "order_id",
        "point",
        "coin_code",
        "gen_time",
        "ext_json",
        "tag_id",
        "store_id",
        "status",
    ]
    .join(",")
});

pub struct FixedH256(pub H256);

impl Display for FixedH256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for i in &self.0 .0 {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

pub async fn persist_one(pool: &Pool, data: &Msg) -> Result<u64, AppError> {
    let client = pool.get().await?;
    let fields = FIELDS.as_str();
    let st =
        format!("INSERT INTO transactions_pool ({fields}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)");
    let rst = client
        .execute(
            &st,
            &[
                &data.from_user_id,
                &data.to_user_id,
                &data.order_id,
                &data.point,
                &data.coin_code,
                &data.gen_time,
                &data.ext_json,
                &data.tag_id,
                &data.store_id,
                &"pending",
            ],
        )
        .await?;
    Ok(rst)
}

pub async fn transfer_token(
    w3: &Arc<Provider<Http>>,
    token_addr: &str,
    from: String,
    from_key: String,
    to: String,
    value: U256,
) -> Result<String, AppError> {
    let chain_id = CHAIN_ID
        .get()
        .unwrap_or_else(|| panic!("get chain_id failed"));
    debug!("chain_id={}", chain_id);
    let from_addr: Address = from.parse()?;
    let contract_addr: Address = token_addr.parse()?;
    let c = Erc20Token::new(contract_addr, w3.clone());

    let wallet = from_key
        .strip_prefix("0x")
        .ok_or(AppError::PrivateKeyError)?
        .parse::<LocalWallet>()?
        .with_chain_id(chain_id.as_u64());
    let data = json!([
        {"jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1},
        {"jsonrpc": "2.0", "method": "eth_getTransactionCount", "params": [from_addr, "latest"], "id": 2},
    ]);

    let rst: [Value; 2] = reqwest::Client::new()
        .post(&SETTING.rpc)
        .json(&data)
        .send()
        .await?
        .json()
        .await?;
    let [gas_price_rsp, nonce_rsp] = rst;
    let tmp = gas_price_rsp["result"]
        .as_str()
        .ok_or(AppError::KeyError("result".into()))?;
    let gas_price = U256::from_str(tmp)?;
    let tmp = nonce_rsp["result"]
        .as_str()
        .ok_or(AppError::KeyError("result".into()))?;
    let nonce = U256::from_str(tmp)?;

    let func_call: FunctionCall<Arc<Provider<Http>>, Provider<Http>, bool> =
        c.transfer(to.parse()?, value);
    let tx_req = func_call
        .gas(50000)
        .gas_price(gas_price)
        .nonce(nonce)
        .from(from_addr);
    let tx: TypedTransaction = tx_req.tx;
    let signature = wallet.sign_transaction(&tx).await?;
    let raw_tx = tx.rlp_signed(&signature);
    let rst = w3.send_raw_transaction(raw_tx).await?;
    // rst.tx_hash().to_string() => 0x7cf4…31e5 produce result with ...
    // override Display trait with FixedH256 get full string without ellipsis
    Ok(rst.tx_hash().encode_hex())
}

pub async fn send_tx(
    w3: &Arc<Provider<Http>>,
    client: &deadpool_postgres::Client,
    msg: &Msg,
) -> Result<String, AppError> {
    let st = "select address,private_key from userinfo where user_id=$1";
    let (from, to) = (&msg.from_user_id, &msg.to_user_id);
    let token_addr = match format!("token_{}", &msg.coin_code).as_str() {
        "token_a" => &SETTING.token_a,
        "token_b" => &SETTING.token_b,
        "token_c" => &SETTING.token_c,
        "token_d" => &SETTING.token_d,
        "token_e" => &SETTING.token_e,
        _ => &SETTING.token_a,
    };

    let from_user = client.query_one(st, &[&from]).await?;
    let to_user = client.query_one(st, &[&to]).await?;

    transfer_token(
        w3,
        token_addr,
        from_user.get::<&str, String>("address"),
        from_user.get::<&str, String>("private_key"),
        to_user.get::<&str, String>("address"),
        U256::from(msg.point),
    )
    .await
}
