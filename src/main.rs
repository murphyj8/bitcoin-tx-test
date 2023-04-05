

use std::time::{Duration, SystemTime};
use std::sync::mpsc;
use average::MeanWithError;

use bitcoin_tx_test::config;
use bitcoin_tx_test::key;

use cpu_time::ThreadTime;
use sv::script::Script;
use sv::messages::{OutPoint,Tx, TxIn, TxOut};
use sv::util::Hash256;
use sv::transaction::sighash::{sighash, SigHashCache, SIGHASH_ALL, SIGHASH_FORKID};
use sv::transaction::generate_signature;
use sv::transaction::p2pkh::{create_unlock_script};
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
fn create_tx(tx_info: Arc<BitcoinTxInfo>) -> f64 {
    let thread_start = ThreadTime::now(); 
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
    let cpu_time = thread_start.elapsed();
    cpu_time.as_nanos() as f64

}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start the timer 
    let now = SystemTime::now();
    
    // load the toml file
    println!("Loading Config toml file {}", "./data/config.toml"); 
    let conf = config::read_config("./data/config.toml").unwrap();

    println!("{:?}", conf);

    let tx_info = Arc::new(BitcoinTxInfo::new(&conf));
    let mut thread_timed_vec = Vec::<f64>::new();

    let loop_count = conf.testparams.iteration_count;
    let mut num_threads =  num_cpus::get_physical(); 
    if conf.testparams.thread_count != 0{
        num_threads = conf.testparams.thread_count as usize;
    }
     // A vector containing all the JoinHandles for the spawned threads
    let mut fetch_handle: Vec<thread::JoinHandle<()>> = Vec::new();
    let total_cpu_start = ThreadTime::now();
    for _ in 0..loop_count{
        let (tx, rx) = mpsc::channel();
        for _ in 0..num_threads{
            let tx_info = Arc::clone(&tx_info);
            let thread_tx = tx.clone();
            fetch_handle.push(thread::spawn(move || {
                //create_tx(tx_info);
                thread_tx.send(create_tx(tx_info)).unwrap();
            }));
        }

        for _ in 0..num_threads{
            thread_timed_vec.push(rx.recv().unwrap());
        }
        while let Some(cur_thread) = fetch_handle.pop() {
            cur_thread.join().unwrap(); 
        } 
    }

    let cpu_total_measured = total_cpu_start.elapsed();
    let a: MeanWithError = thread_timed_vec.iter().copied().collect();

    println!("Average CPU time per tx {:?} for {} transactions", 
                Duration::from_nanos(a.mean() as u64),
                num_threads as i32 * loop_count
            );
    println!("Total CPU time for {} transactions is {:?} (sum of each thread time)", 
                num_threads as i32 * loop_count, 
                Duration::from_nanos(thread_timed_vec.iter().sum::<f64>() as u64)
            ); 
    println!("Total CPU time for the process for {} transactions is {:?}",
                num_threads as i32 * loop_count, 
                cpu_total_measured); 
    println!("Total Wall-clock Time for the process for {} transactions is {:?}", 
                num_threads as i32 * loop_count, 
                now.elapsed().unwrap()); 

    println!("Finishing");
    Ok(())
}

/*
    write the file info
 */