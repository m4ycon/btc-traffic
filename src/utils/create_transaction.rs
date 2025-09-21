use bitcoin::{Address, Amount, Sequence, Transaction};
use corepc_node::{Client, Input, Output};
use serde::Deserialize;
use std::error::Error;

const DEFAULT_FEE: u64 = 1000; // in satoshis

// TODO: Remove this when corepc-types is updated
#[derive(Debug, Deserialize)]
pub struct UnspentOutput {
    pub txid: String,
    pub vout: i64,
    pub address: String,
    pub label: String,
    // pub account: String, // this throws an error in original code
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: String,
    pub amount: f64,
    pub confirmations: i64,
    #[serde(rename = "redeemScript")]
    pub redeem_script: Option<String>,
    pub spendable: bool,
    pub solvable: bool,
    pub safe: bool,
}

pub async fn create_self_transactions(
    client: &Client,
    to_address: &Address,
) -> Result<Transaction, Box<dyn Error>> {
    let transactions = create_many_self_transactions(client, to_address, 1).await.unwrap();
    Ok(transactions[0].clone())
}

pub async fn create_many_self_transactions(
    client: &Client,
    to_address: &Address,
    count: usize,
) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut transactions = Vec::with_capacity(count);

    let unspent: Vec<UnspentOutput> = client.call("listunspent", &[])?;
    for (i, utxo) in unspent.iter().enumerate() {
        if i >= count {
            break;
        }

        let amount = Amount::from_btc(utxo.amount).unwrap() - Amount::from_sat(DEFAULT_FEE);
        println!("Creating transfer of {} to {}", amount, to_address);
        let tx = create_transaction(client, to_address, utxo, amount.clone())?;

        transactions.push(tx);
    }

    Ok(transactions)
}

pub fn create_transaction(
    client: &Client,
    to_address: &Address,
    utxo: &UnspentOutput,
    amount: Amount,
) -> Result<Transaction, Box<dyn Error>> {
    let inputs = {
        vec![Input {
            txid: utxo.txid.parse()?,
            vout: u64::try_from(utxo.vout)?,
            sequence: Some(Sequence::MAX),
        }]
    };
    let outputs = [Output::new(to_address.clone(), amount)];

    let raw_tx = client.create_raw_transaction(&inputs, &outputs)?;

    let signed_tx = client.sign_raw_transaction_with_wallet(&raw_tx.transaction()?)?;
    if !signed_tx.complete {
        return Err("Failed to sign transaction".into());
    }

    if signed_tx.hex.is_empty() {
        return Err("Signed transaction hex is empty".into());
    }
    let tx_bytes = hex::decode(&signed_tx.hex).map_err(|e| format!("Hex decode error: {}", e))?;
    if tx_bytes.is_empty() {
        return Err("Decoded transaction bytes are empty".into());
    }

    let final_tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes)
        .map_err(|e| format!("Deserialization error: {}", e))?;

    Ok(final_tx)
}
