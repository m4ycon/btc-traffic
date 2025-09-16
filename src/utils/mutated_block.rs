use bitcoin::blockdata::block::Block;
use bitcoin::{opcodes::all::OP_RETURN, script::Builder, Address};
use corepc_node::{Client, Error};

use crate::utils::{create_block::create_block, create_transaction::create_transaction};

pub async fn create_mutated_block_1(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let self_transfer = create_transaction(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    // overwrite coinbase scriptPubKey to OP_RETURN, removing witness commitment
    let coinbase = block.txdata.first_mut().unwrap();
    coinbase.output[0].script_pubkey = Builder::new().push_opcode(OP_RETURN).into_script();

    // recompute merkle root
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    let block_hash = block.block_hash();
    println!("Mutated block hash: {}", block_hash);

    Ok(block)
}
