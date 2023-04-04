use bitcoin::util::key::{PrivateKey, PublicKey}; 

use sv::transaction::sighash::{sighash, SigHashCache, SIGHASH_ALL, SIGHASH_FORKID};
use sv::transaction::generate_signature;
use sv::util::{Hash160, Hash256, Result};
use sv::script::Script;
use sv::transaction::p2pkh::{create_lock_script, create_unlock_script};
use sv::messages::{Tx, TxIn, TxOut, OutPoint, Payload};
use sv::util::Serializable;

use std::io::Cursor;
use super::util::decode_hexstr;

/// Convert a transaction into a hexstring
pub fn tx_as_hexstr(tx: &Tx) -> String {
    let mut b = Vec::with_capacity(tx.size());
    tx.write(&mut b).unwrap();
    hex::encode(&b)
}

pub fn get_tx_id(tx_str: String) -> Hash256{
    let tx_bytes = decode_hexstr(&tx_str).unwrap();
    let tx_var = Tx::read(&mut Cursor::new(&tx_bytes)).unwrap();
    tx_var.hash()
}

/// convert an output into a hexstring
pub fn txout_as_hexstr(txout: &TxOut) -> String{
    let mut out = Vec::with_capacity(txout.size());
    txout.write(&mut out).unwrap();
    hex::encode(&out)
}

pub fn create_tx_vin(mut tx: Tx) -> Tx{
    tx
}

pub fn create_tx_out(sats_amt: i64, script_pubkey_bytes: Vec<u8>) -> TxOut{
    let mut payment_script_pubkey = Script::new(); 
    payment_script_pubkey.append_slice(&script_pubkey_bytes);

    TxOut{satoshis: sats_amt, lock_script: payment_script_pubkey.clone()}
}

pub fn add_signature_for_tx_index(tx: &mut Tx, tx_index: usize, signature: &Vec<u8>, public_key: PublicKey){
    tx.inputs[tx_index].unlock_script =
        create_unlock_script(&signature, &public_key.to_bytes().try_into().unwrap());
}

pub fn tx_sighash(tx: &Tx, tx_index: usize, script_bytes: &[u8], sats: i64, sighash_type: u8) -> Result<Hash256>{
    let mut cache = SigHashCache::new();
    sighash(
        &tx,
        tx_index,
        //&change_script_pub_key_bytes.0,
        &script_bytes,
        sats,
        sighash_type,
        &mut cache,
    )
}
pub fn sign_tx(priv_key: PrivateKey, sighash: Hash256, sighash_type: u8) -> Vec<u8> {

    let signature = generate_signature(
        &priv_key.to_bytes().try_into().unwrap(),
        &sighash,
        sighash_type,
    );
    signature.unwrap()
}


