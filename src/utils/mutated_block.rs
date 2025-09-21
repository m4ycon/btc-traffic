use bitcoin::blockdata::block::Block;
use bitcoin::hashes::Hash;
use bitcoin::{opcodes::all::OP_RETURN, script::Builder, Address};
use corepc_node::{Client, Error};

use crate::utils::create_block::{mine_block, update_merkle_root};
use crate::utils::create_transaction::{create_many_self_transactions, create_self_transactions};
use crate::utils::{create_block::create_block};

pub async fn create_mutated_block_1(client: &Client, to_address: &Address) -> Result<Block, Error> {
    // just logged by btc-core in debug=validation mode
    let block = create_mutated_block(client, to_address, "bad-txnmrklroot", |block| {
        let merkle_root = block.header.merkle_root.clone();
        let mut bytes = *merkle_root.as_raw_hash().as_byte_array();
        bytes[0] ^= 0x55;
        block.header.merkle_root = Hash::from_byte_array(bytes);
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

pub async fn create_mutated_block_2(client: &Client, to_address: &Address) -> Result<Block, Error> {
    // cant submit it even with mutated block errors commented, as it still fails with "bad-txns-inputs-missingorspent"
    let self_transfers = create_many_self_transactions(client, to_address, 2).await.unwrap();

    let valid_block = create_block(
        &client,
        Some(vec![
            self_transfers[0].clone(),
            self_transfers[1].clone(),
        ])
    ).unwrap();

    let mutated_block = create_block(
        &client,
        Some(vec![
            self_transfers[0].clone(),
            self_transfers[1].clone(),
            self_transfers[1].clone(),
        ])
    ).unwrap();

    assert!(valid_block.header.merkle_root == mutated_block.header.merkle_root);

    println!("Mutated block hash ({}): {}", "bad-txns-duplicate", valid_block.block_hash());

    Ok(mutated_block)
}

pub async fn create_mutated_block_3(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, "bad-witness-nonce-size", |block| {
        // add extra item to witness stack
        let coinbase = block.txdata.first_mut().unwrap();
        coinbase.input[0].witness.push([0]);
        update_merkle_root(block);
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

pub async fn create_mutated_block_4(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, "bad-witness-merkle-match", |block| {
        // change commitment in coinbase to invalid value
        let coinbase = block.txdata.first_mut().unwrap();
        let script_pubkey_bytes = coinbase.output[0].script_pubkey.as_mut_bytes();
        script_pubkey_bytes[script_pubkey_bytes.len() - 1] ^= 0x01;
        update_merkle_root(block);
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

pub async fn create_mutated_block_5(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, "unexpected-witness", |block| {
        // overwrite coinbase scriptPubKey to OP_RETURN, removing witness commitment
        let coinbase = block.txdata.first_mut().unwrap();
        coinbase.output[0].script_pubkey = Builder::new().push_opcode(OP_RETURN).into_script();
        update_merkle_root(block);
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

async fn create_mutated_block(
    client: &Client,
    to_address: &Address,
    mutate_message: &str,
    mutate_callback: fn(&mut Block) -> Result<(), Error>,
) -> Result<Block, Error> {
    let self_transfer = create_self_transactions(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    mutate_callback(&mut block)?;

    mine_block(&mut block);

    let block_hash = block.block_hash();
    println!("Mutated block hash ({}): {}", mutate_message, block_hash);

    Ok(block)
}
