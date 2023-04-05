
//use serde::{Deserialize, Serialize};
#![allow(dead_code)]
use serde_derive::{Deserialize, Serialize}; 
use sv::messages::Tx;
use sv::util::Serializable;
use std::io::Cursor; 

use crate::util::decode_hexstr;
/// Structure for json serialisation for broadcast_tx
#[derive(Debug, Serialize)]
pub struct BroadcastTxType {
    pub txhex: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainInfo{
    chain: String,
    blocks: i32,
    headers: i32, 
    bestblockhash: String,
    difficulty: f64,
    mediantime: i32,
    verificationprogress: f32,
    pruned: bool,
    chainwork: String, 
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct AddressInfo{
    pub address: String, 
    ismine: bool,
    isscript: bool, 
    isvalid: bool,
    iswatchonly: bool, 
    pub scriptPubKey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressBalance{
    confirmed: u64,
    unconfirmed: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AddressUTXO{
    height: u32, 
    pub tx_pos: u32,
    pub tx_hash: String, 
    pub value: u64
}
pub type UtxoSet = Vec<AddressUTXO>;


pub async fn health(network: &str) -> Result<String, Box<dyn std::error::Error>>{
    let url = 
        format!("https://api.whatsonchain.com/v1/bsv/{}/woc", network); 
    let chain_info_resp = reqwest::get(url).await?.text().await?;
    Ok(chain_info_resp)
}


pub async fn chaininfo(network: &str) -> Result<ChainInfo, Box<dyn std::error::Error>>{
    let url = 
        format!("https://api.whatsonchain.com/v1/bsv/{}/chain/info", network); 
    let chain_info_resp = reqwest::get(url).await?; 
    let chain_info = chain_info_resp.json::<ChainInfo>().await?;
    Ok(chain_info)
}

pub async fn addrinfo(network: &str, address: &str) -> Result<AddressInfo, Box<dyn std::error::Error>>{
    let url = 
        format!("https://api.whatsonchain.com/v1/bsv/{}/address/{}/info", network, address);
    let addr_info_resp = reqwest::get(url).await?;
    let addr_info = addr_info_resp.json::<AddressInfo>().await?;
    Ok(addr_info)
}

pub async fn balance(network: &str, address: &str) -> Result<AddressBalance, Box<dyn std::error::Error>>{
    let url = 
        format!("https://api.whatsonchain.com/v1/bsv/{}/address/{}/balance", network, address);
    let balance_resp = reqwest::get(url).await?;
    let balance_info = balance_resp.json::<AddressBalance>().await?;
    Ok(balance_info)
}


pub async fn addrutxo(network: &str, address: &str) -> Result<UtxoSet, Box<dyn std::error::Error>>{
    let url =
        format!("https://api.whatsonchain.com/v1/bsv/{}/address/{}/unspent", network, address); 
    let resp = reqwest::get(url).await?;
    let utxoset = resp.json::<UtxoSet>().await?;
    Ok(utxoset)
}

pub async fn rawtx(network: &str, tx_hash: &str) -> Result<Tx, Box<dyn std::error::Error>>{
    let url = 
        format!("https://api.whatsonchain.com/v1/bsv/{}/tx/{}/hex", network, tx_hash);
    let tx_raw_resp = reqwest::get(url).await?;
    let tx_raw = tx_raw_resp.text().await?;

    let tx_bytes = decode_hexstr(&tx_raw).unwrap();
    let tx: Tx = Tx::read(&mut Cursor::new(&tx_bytes)).unwrap(); 
    Ok(tx)
}

pub async fn broadcast_tx(network: &str, tx_hex: &str) -> Result<String, reqwest::Error> {
    let url =
        format!("https://api.whatsonchain.com/v1/bsv/{network}/tx/raw");
    
    let data_for_broadcast = BroadcastTxType {
        txhex: tx_hex.to_string(),
    };

    //let data = serde_json::to_string(&data_for_broadcast).unwrap();
    println!{"{:?}", data_for_broadcast};
    let client = reqwest::Client::new(); 
    let res = client.post(url)
                .json(&data_for_broadcast)
                .send()
                .await
                .expect("failed to get response")
                .text()
                .await
                .expect("failed to get payload");
    Ok(res)
}

