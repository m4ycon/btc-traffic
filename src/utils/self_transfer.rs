use std::error::Error;
use bitcoin::{Address, Amount, Transaction, Txid};
use corepc_node::{Client, Input, Output};
use serde::Deserialize;

// TODO: Remove this when corepc-types is updated
#[derive(Debug, Deserialize)]
struct UnspentOutput {
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

#[derive(Debug)]
pub struct SelfTransferResult {
    pub txid: Txid,
    pub address: Address,
}

/// Inspired by Bitcoin Core `create_self_transfer`
/// https://github.com/bitcoin/bitcoin/blob/master/test/functional/test_framework/wallet.py#L360
pub async fn create_self_transfer(
    client: &Client,
    address: Option<&Address>,
) -> Result<SelfTransferResult, Box<dyn Error>> {
    // Step 1: Generate a new address in the wallet if none provided
    let address = match address {
        Some(addr) => addr.clone(),
        None => client
            .get_new_address(None, None)?
            .address()?
            .assume_checked(),
    };


    // Step 2: Prepare inputs for the raw transaction
    let unspent: Vec<UnspentOutput> = client.call("listunspent", &[])?;
    let utxo = unspent.iter().find(|u| u.address == address.to_string()).unwrap();
    let fee = 0.0000_1000;
    let amount = Amount::from_btc(utxo.amount - fee).unwrap();
    println!("Sending {} to {}", amount, address);

    let inputs = {
        vec![Input {
            txid: utxo.txid.parse()?,
            vout: u64::try_from(utxo.vout)?,
            sequence: None,
        }]
    };
    let outputs = [
        Output::new(address.clone(), amount),
    ];

    // Step 3: Create raw transaction
    let raw_tx = client.create_raw_transaction(&inputs, &outputs)?;
    println!("Raw transaction: {:?}", raw_tx);

    // Step 4: Sign the transaction
    let signed_tx = client.sign_raw_transaction_with_wallet(&raw_tx.transaction()?)?;
    println!("Signed transaction hex: {}", signed_tx.hex);
    if !signed_tx.complete {
        return Err("Failed to sign transaction".into());
    }

    // Step 5: Validate hex string
    if signed_tx.hex.is_empty() {
        return Err("Signed transaction hex is empty".into());
    }
    let tx_bytes = hex::decode(&signed_tx.hex).map_err(|e| format!("Hex decode error: {}", e))?;
    if tx_bytes.is_empty() {
        return Err("Decoded transaction bytes are empty".into());
    }

    // Step 6: Deserialize transaction
    let final_tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes)
        .map_err(|e| format!("Deserialization error: {}", e))?;

    // Step 7: Send the transaction
    let txid = client.send_raw_transaction(&final_tx)?.txid()?;

    Ok(SelfTransferResult { txid, address })
}
