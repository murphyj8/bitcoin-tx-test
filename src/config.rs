use serde_derive::{Deserialize, Serialize};
use toml;
use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct Wallet{
    pub private_key_for_instance: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TestParams{
    pub thread_count: i32,
    pub iteration_count: i32
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Service{
    pub network: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TransactionInputs{
    pub tx_hash: String,
    pub tx_pos: u32, 
    pub amount: i64
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TransactionOutputs{
    pub scriptpubkey: String,
    pub amount: u64
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config{
    pub wallet: Wallet,
    pub testparams: TestParams,
    pub service: Service,
    pub transactioninputs: TransactionInputs,
    pub transactionoutputs: TransactionOutputs
}

pub fn read_config(filename: &str) -> std::io::Result<Config>{
    let contents = match fs::read_to_string(filename){
        Ok(c) => c, 
        Err(error) =>{
            panic!("Could nit read config file -> {} with error -> {}", filename, error);
        }
    };

    let config: Config = match toml::from_str(&contents){
        Ok(config) => config, 
        Err(error) => {
            panic!("Could not read config because of error {}", error);
        }
    };
    Ok(config)
}