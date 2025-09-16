use bitcoin::blockdata::block::Block;
use bitcoin::{opcodes::all::OP_RETURN, script::Builder, Address};
use corepc_node::{Client, Error};

use crate::utils::create_block::{mine_block, update_merkle_root};
use crate::utils::{create_block::create_block, create_transaction::create_transaction};

// bad-txnmrklroot
pub async fn create_mutated_block_1(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, |block| {
        todo!()
    })
    .await
    .unwrap();

    Ok(block)
}

// bad-txns-duplicate
pub async fn create_mutated_block_2(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, |block| {
        todo!()
    })
    .await
    .unwrap();

    Ok(block)
}

// bad-witness-nonce-size
pub async fn create_mutated_block_3(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, |block| {
        todo!()
    })
    .await
    .unwrap();

    Ok(block)
}

// bad-witness-merkle-match
pub async fn create_mutated_block_4(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, |block| {
        // change commitment in coinbase to invalid value
        let coinbase = block.txdata.first_mut().unwrap();
        let script_pubkey_bytes = coinbase.output[0].script_pubkey.as_mut_bytes();
        script_pubkey_bytes[script_pubkey_bytes.len() - 1] ^= 0x01;
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

// unexpected-witness
pub async fn create_mutated_block_5(client: &Client, to_address: &Address) -> Result<Block, Error> {
    let block = create_mutated_block(client, to_address, |block| {
        // overwrite coinbase scriptPubKey to OP_RETURN, removing witness commitment
        let coinbase = block.txdata.first_mut().unwrap();
        coinbase.output[0].script_pubkey = Builder::new().push_opcode(OP_RETURN).into_script();
        Ok(())
    })
    .await
    .unwrap();

    Ok(block)
}

async fn create_mutated_block(
    client: &Client,
    to_address: &Address,
    mutate_callback: fn(&mut Block) -> Result<(), Error>,
) -> Result<Block, Error> {
    let self_transfer = create_transaction(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    mutate_callback(&mut block)?;

    finalize_block(&mut block);

    let block_hash = block.block_hash();
    println!("Mutated block hash: {}", block_hash);

    Ok(block)
}

fn finalize_block(block: &mut Block) {
    update_merkle_root(block);
    mine_block(block);
}
