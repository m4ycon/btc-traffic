use bitcoin::blockdata::block::Block;
use bitcoin::{opcodes::all::OP_RETURN, script::Builder, Address};
use corepc_node::{Client, Error};

use crate::utils::{create_block::create_block, create_transaction::create_transaction};

// bad-txnmrklroot
pub async fn create_mutated_block_1(client: &Client, to_address: &Address) -> Result<Block, Error> {
    todo!()
}

// bad-txns-duplicate
pub async fn create_mutated_block_2(client: &Client, to_address: &Address) -> Result<Block, Error> {
    todo!()
}

// bad-witness-nonce-size
pub async fn create_mutated_block_3(client: &Client, to_address: &Address) -> Result<Block, Error> {
    todo!()
}

// bad-witness-merkle-match
pub async fn create_mutated_block_4(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let self_transfer = create_transaction(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    // change commitment in coinbase to invalid value
    let coinbase = block.txdata.first_mut().unwrap();
    let script_pubkey_bytes = coinbase.output[0].script_pubkey.as_mut_bytes();
    script_pubkey_bytes[script_pubkey_bytes.len() - 1] ^= 0x01;

    // recompute merkle root
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    // Compute target
    let target = block.header.target();
    while !block.header.validate_pow(target).is_ok() {
        block.header.nonce += 1;
    }

    let block_hash = block.block_hash();
    println!("Mutated block hash (bad-witness-merkle-match): {}", block_hash);

    Ok(block)
}

// unexpected-witness
pub async fn create_mutated_block_5(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let self_transfer = create_transaction(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    // overwrite coinbase scriptPubKey to OP_RETURN, removing witness commitment
    let coinbase = block.txdata.first_mut().unwrap();
    coinbase.output[0].script_pubkey = Builder::new().push_opcode(OP_RETURN).into_script();

    // recompute merkle root
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    let block_hash = block.block_hash();
    println!("Mutated block hash (unexpected-witness): {}", block_hash);

    Ok(block)
}
