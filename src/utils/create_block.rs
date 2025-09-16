use bitcoin::block::Version;
use bitcoin::blockdata::block::Block;
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::hash_types::BlockHash;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::opcodes::all::OP_RETURN;
use bitcoin::script::{Builder, PushBytesBuf};
use bitcoin::{CompactTarget, TxMerkleNode};
use corepc_node::{Client, Error};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::create_coinbase::create_coinbase;

// Constants from blocktools.py
// https://github.com/bitcoin/bitcoin/blob/dadf15f88cbad37538d85415ae5da12d4f0f1721/test/functional/test_framework/blocktools.py
const VERSIONBITS_LAST_OLD_BLOCK_VERSION: i32 = 4;
const REGTEST_DIFFICULTY: u32 = 0x207fffff;
const COMMITMENT_HEADER: [u8; 4] = [0xaa, 0x21, 0xa9, 0xed];

pub fn create_block(
    client: &Client,
    txlist: Option<Vec<Transaction>>,
) -> Result<Block, Error> {
    // Create coinbase transaction if not provided
    let height = (get_block_height(client) + 1) as u32;
    let coinbase = create_coinbase(height);

    // Collect transactions
    let mut transactions = vec![coinbase];
    if let Some(txlist) = txlist {
        transactions.extend(txlist);
    }

    // Initialize block header
    let header = bitcoin::block::Header {
        version: Version::from_consensus(VERSIONBITS_LAST_OLD_BLOCK_VERSION),
        prev_blockhash: get_prev_hash(client),
        merkle_root: TxMerkleNode::all_zeros(), // fill in later
        time: get_min_timestamp(client),
        bits: CompactTarget::from_consensus(REGTEST_DIFFICULTY),
        nonce: 0, // Set to 0 for now; adjust if mining is needed
    };

    // Create and return the block
    let mut block = Block {
        header: header,
        txdata: transactions,
    };

    prepare_commitment(&mut block);
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    // Compute target
    let target = block.header.target();
    while !block.header.validate_pow(target).is_ok() {
        block.header.nonce += 1;
    }

    Ok(block)
}

fn prepare_commitment(block: &mut Block) -> () {
    let commitment = {
        let witness_root = block.witness_root().unwrap();
        let mut data = Vec::from(witness_root.to_byte_array());
        data.extend_from_slice(&[0u8; 32]); // reserved value
        sha256d::Hash::hash(&data).to_byte_array()
    };

    let mut witness_bytes = Vec::with_capacity(4 + commitment.len());
    witness_bytes.extend(COMMITMENT_HEADER);
    witness_bytes.extend(commitment);

    let coinbase = block.txdata.first_mut().unwrap();
    coinbase.output[0].script_pubkey = Builder::new()
        .push_opcode(OP_RETURN)
        .push_slice(PushBytesBuf::try_from(witness_bytes).unwrap())
        .into_script();
}

fn get_min_timestamp(client: &Client) -> u32 {
    let blockchain_info = client.get_blockchain_info().unwrap();
    let min_timestamp = blockchain_info.median_time as u32 + 1;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    if now > min_timestamp {
        return now;
    }
    min_timestamp
}

fn get_prev_hash(client: &Client) -> BlockHash {
    client.get_best_block_hash().unwrap().block_hash().unwrap()
}

fn get_block_height(client: &Client) -> u64 {
    client.get_block_count().unwrap().0
}
