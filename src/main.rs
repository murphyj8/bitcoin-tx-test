

use std::io::Cursor;
use bitcoin_tx_test::util::decode_hexstr;
use bitcoin_tx_test::config;
use bitcoin_tx_test::key;

use sv::script::Script;
use sv::util::Serializable; 
use sv::messages::{OutPoint,Tx, TxIn, TxOut, Payload};
use sv::util::{Hash160, Hash256};
use sv::transaction::sighash::{sighash, SigHashCache, SIGHASH_ALL, SIGHASH_FORKID};
use sv::transaction::generate_signature;
use sv::transaction::p2pkh::{create_lock_script, create_unlock_script};
//use threadpool::ThreadPool;
use num_cpus; 
use std::thread;
use std::sync::Arc;

mod util;
mod restapi;
mod bitcoin_if;

#[derive(Debug, Default)]
pub struct BitcoinTxInfo{
    pub tx_pos: u32,
    pub tx_hash: String, 
    pub script_pub_key:  Script,
    pub script_sig: Script, 
    pub amt_sats: u64,
    pub priv_key: key::KeyInfo
}

impl BitcoinTxInfo{
    pub fn new(conf: &config::Config) -> BitcoinTxInfo{
        let mut tx = BitcoinTxInfo{ tx_hash: conf.transactioninputs.tx_hash.clone(),
                        tx_pos: conf.transactioninputs.tx_pos,
                        script_pub_key: Script::new(),
                        script_sig: Script::new(),
                        amt_sats: conf.transactionoutputs.amount,
                        priv_key: key::KeyInfo::new(&conf)
        };

        let change_script_pub_key_bytes = hex::decode(&conf.transactionoutputs.scriptpubkey).unwrap(); 
        tx.script_pub_key.append_slice(&change_script_pub_key_bytes);
        tx
    }
}
fn create_tx(tx_info: Arc<BitcoinTxInfo>){
    //println!("{}", tx_info.priv_key.get_p2pkh());
    let vins: Vec<TxIn> = vec![TxIn {
        prev_output: OutPoint {
            hash: Hash256::decode(&tx_info.tx_hash).unwrap(),
            index: tx_info.tx_pos,
        },
        unlock_script: Script::new(),
        sequence: 0xffffffff,
    }]; 

    let vouts: Vec<TxOut> = vec![
        TxOut {
            satoshis: tx_info.amt_sats as i64,
            lock_script: tx_info.script_pub_key.clone()
        }];

    let mut tx = Tx {
        version: 1,
        inputs: vins,
        outputs: vouts,
        lock_time: 0,
    };

       // Sign transaction
       let mut cache = SigHashCache::new();
       let sighash_type = SIGHASH_ALL | SIGHASH_FORKID;
   
       let sighash = sighash(
           &tx,
           0,
           //&change_script_pub_key_bytes.0,
           &tx_info.script_pub_key.0,
           tx_info.amt_sats as i64,
           sighash_type,
           &mut cache,
       )
       .unwrap();
   
       let signature = generate_signature(
           &tx_info.priv_key.get_private_key().to_bytes().try_into().unwrap(),
           //&self.private_key.to_bytes().try_into().unwrap(),
           &sighash,
           sighash_type,
       )
       .unwrap();
   
       tx.inputs[0].unlock_script =
               create_unlock_script(&signature, &tx_info.priv_key.get_public_key().to_bytes().try_into().unwrap());
   
       //dbg!(tx);
       //let tx_hex = bitcoin_if::tx_as_hexstr(&tx);
       //println!("{}", tx_hex);

}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let tx_string = "0100000001813f79011acb80925dfe69b3def355fe914bd1d96a3f5f71bf8303c6a989c7d1000000006b483045022100ed81ff192e75a3fd2304004dcadb746fa5e24c5031ccfcf21320b0277457c98f02207a986d955c6e0cb35d446a89d3f56100f4d7f67801c31967743a9c8e10615bed01210349fc4e631e3624a545de3f89f5d8684c7b8138bd94bdd531d2e213bf016b278afeffffff02a135ef01000000001976a914bc3b654dca7e56b04dca18f2566cdaf02e8d9ada88ac99c39800000000001976a9141c4bc762dd5423e332166702cb75f40df79fea1288ac19430600";

    let bytes = decode_hexstr(tx_string).unwrap();

    let tx_var : Tx = Tx::read(&mut Cursor::new(&bytes)).unwrap();

    println!("{:?}", tx_var);

    // load the toml file
    println!("Loading Config toml file {}", "./data/config.toml"); 
    let conf = config::read_config("./data/config.toml").unwrap();

    println!("{:?}", conf);

    let tx_info = Arc::new(BitcoinTxInfo::new(&conf));

    let loop_count = conf.testparams.iteration_count;
    let num_threads =  num_cpus::get_physical(); 
     // A vector containing all the JoinHandles for the spawned threads
    let mut fetch_handle: Vec<thread::JoinHandle<()>> = Vec::new();
    for _ in 0..loop_count{
        for _ in 0..num_threads{
            let tx_info = Arc::clone(&tx_info);
            fetch_handle.push(thread::spawn(move || {
                create_tx(tx_info);
            }));
        }

        while let Some(cur_thread) = fetch_handle.pop() {
            cur_thread.join().unwrap();
        }
    }
    println!("Finishing");
    Ok(())
}

/*
    lanuch a thread for each cpu (if the thread_count is zero)
    loop for each iteration
    add the timing details
    write the file info
 */