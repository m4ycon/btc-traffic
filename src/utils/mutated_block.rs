use bitcoin::blockdata::block::Block;
use bitcoin::consensus::serialize;
use bitcoin::hashes::Hash;
use bitcoin::{opcodes::all::OP_RETURN, script::Builder, Address};
use corepc_node::{Client, Error};

use crate::utils::create_block::create_block;
use crate::utils::create_block::{mine_block, update_merkle_root};
use crate::utils::create_transaction::{create_many_self_transactions, create_self_transactions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutatedBlockError {
    BadTxnMrklRoot,
    BadTxnsDuplicate,
    BadWitnessNonceSize,
    BadWitnessMerkleMatch,
    UnexpectedWitness,
}

impl MutatedBlockError {
    pub fn as_str(&self) -> &'static str {
        match self {
            MutatedBlockError::BadTxnMrklRoot => "bad-txnmrklroot",
            MutatedBlockError::BadTxnsDuplicate => "bad-txns-duplicate",
            MutatedBlockError::BadWitnessNonceSize => "bad-witness-nonce-size",
            MutatedBlockError::BadWitnessMerkleMatch => "bad-witness-merkle-match",
            MutatedBlockError::UnexpectedWitness => "unexpected-witness",
        }
    }

    pub async fn create_mutated_block(
        &self,
        client: &Client,
        to_address: &Address,
    ) -> Result<Block, Error> {
        match self {
            MutatedBlockError::BadTxnMrklRoot => {
                create_bad_txn_mrkl_root_block(client, to_address).await
            }

            MutatedBlockError::BadTxnsDuplicate => {
                create_bad_txns_duplicate_block(client, to_address).await
            }

            MutatedBlockError::BadWitnessNonceSize => {
                create_bad_witness_nonce_size_block(client, to_address).await
            }

            MutatedBlockError::BadWitnessMerkleMatch => {
                create_bad_witness_merkle_match_block(client, to_address).await
            }

            MutatedBlockError::UnexpectedWitness => {
                create_unexpected_witness_block(client, to_address).await
            }
        }
    }

    pub async fn print_mutated_block_raw_hash(
        &self,
        client: &Client,
        to_address: &Address,
    ) -> Result<(), String> {
        let mutated_block = self
            .create_mutated_block(client, to_address)
            .await
            .map_err(|e| format!("Failed to create mutated block: {}", e))?;

        println!(
            "Block raw hex ({}): {}",
            self.as_str(),
            hex::encode(serialize(&mutated_block))
        );
        println!("You can use 'bitcoin-cli -regtest submitblock <hex>' to submit it.");
        Ok(())
    }
}

async fn create_bad_txn_mrkl_root_block(
    client: &Client,
    to_address: &Address,
) -> Result<Block, Error> {
    create_mutated_block(client, to_address, |block| {
        let merkle_root = block.header.merkle_root.clone();
        let mut bytes = *merkle_root.as_raw_hash().as_byte_array();
        bytes[0] ^= 0x55;
        block.header.merkle_root = Hash::from_byte_array(bytes);
        Ok(())
    })
    .await
}

async fn create_bad_txns_duplicate_block(
    client: &Client,
    to_address: &Address,
) -> Result<Block, Error> {
    // cant submit it even with mutated block errors commented, as it still fails with "bad-txns-inputs-missingorspent"
    let self_transfers = create_many_self_transactions(client, to_address, 2)
        .await
        .unwrap();

    let valid_block = create_block(
        &client,
        Some(vec![self_transfers[0].clone(), self_transfers[1].clone()]),
    )
    .unwrap();

    let mutated_block = create_block(
        &client,
        Some(vec![
            self_transfers[0].clone(),
            self_transfers[1].clone(),
            self_transfers[1].clone(),
        ]),
    )
    .unwrap();

    assert!(valid_block.header.merkle_root == mutated_block.header.merkle_root);

    Ok(mutated_block)
}

async fn create_bad_witness_nonce_size_block(
    client: &Client,
    to_address: &Address,
) -> Result<Block, Error> {
    create_mutated_block(client, to_address, |block| {
        // add extra item to witness stack
        let coinbase = block.txdata.first_mut().unwrap();
        coinbase.input[0].witness.push([0]);
        update_merkle_root(block);
        Ok(())
    })
    .await
}

async fn create_bad_witness_merkle_match_block(
    client: &Client,
    to_address: &Address,
) -> Result<Block, Error> {
    create_mutated_block(client, to_address, |block| {
        // change commitment in coinbase to invalid value
        let coinbase = block.txdata.first_mut().unwrap();
        let script_pubkey_bytes = coinbase.output[0].script_pubkey.as_mut_bytes();
        script_pubkey_bytes[script_pubkey_bytes.len() - 1] ^= 0x01;
        update_merkle_root(block);
        Ok(())
    })
    .await
}

async fn create_unexpected_witness_block(
    client: &Client,
    to_address: &Address,
) -> Result<Block, Error> {
    create_mutated_block(client, to_address, |block| {
        // overwrite coinbase scriptPubKey to OP_RETURN, removing witness commitment
        let coinbase = block.txdata.first_mut().unwrap();
        coinbase.output[0].script_pubkey = Builder::new().push_opcode(OP_RETURN).into_script();
        update_merkle_root(block);
        Ok(())
    })
    .await
}

async fn create_mutated_block(
    client: &Client,
    to_address: &Address,
    mutate_callback: fn(&mut Block) -> Result<(), Error>,
) -> Result<Block, Error> {
    let self_transfer = create_self_transactions(client, to_address).await.unwrap();

    let mut block = create_block(&client, Some(vec![self_transfer.clone()])).unwrap();

    mutate_callback(&mut block)?;

    mine_block(&mut block);

    Ok(block)
}
