use bitcoin::block::Version;
use bitcoin::blockdata::block::{Block};
use bitcoin::blockdata::transaction::{Transaction};
use bitcoin::hash_types::{BlockHash};
use bitcoin::hashes::{Hash};
use bitcoin::{CompactTarget, TxMerkleNode};
use corepc_node::{serde_json, Error};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::create_coinbase::{create_coinbase};

// Constants from blocktools.py
// https://github.com/bitcoin/bitcoin/blob/dadf15f88cbad37538d85415ae5da12d4f0f1721/test/functional/test_framework/blocktools.py
const VERSIONBITS_LAST_OLD_BLOCK_VERSION: i32 = 4;
const REGTEST_DIFFICULTY: u32 = 0x207fffff;
const COIN: u64 = 1_0000_0000; // 1 BTC in satoshis
const COINBASE_MATURITY: u32 = 100;

pub fn create_block(
    hashprev: Option<BlockHash>,
    coinbase: Option<Transaction>,
    ntime: Option<u32>,
    version: Option<Version>,
    tmpl: Option<serde_json::Value>,
    txlist: Option<Vec<Transaction>>,
) -> Result<Block, Error> {
    // Initialize block header
    let header = bitcoin::block::Header {
        version: version.unwrap_or_else(|| {
            tmpl.as_ref()
                .and_then(|t| t.get("version").and_then(|v| v.as_i64()).map(|v| Version::from_consensus(v as i32)))
                .unwrap_or(Version::from_consensus(VERSIONBITS_LAST_OLD_BLOCK_VERSION))
        }),
        prev_blockhash: hashprev.unwrap_or_else(|| {
            tmpl.as_ref()
                .and_then(|t| t.get("previousblockhash").and_then(|h| h.as_str()))
                .and_then(|h| h.parse::<BlockHash>().ok())
                .unwrap_or_else(|| BlockHash::all_zeros())
        }),
        merkle_root: TxMerkleNode::all_zeros(), // Set to 0 for now
        time: ntime.unwrap_or_else(|| {
            tmpl.as_ref()
                .and_then(|t| t.get("curtime").and_then(|t| t.as_u64()).map(|t| t as u32))
                .unwrap_or_else(|| {
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u32
                        + 600
                })
        }),
        bits: tmpl
            .as_ref()
            .and_then(|t| t.get("bits").and_then(|b| b.as_str()))
            .and_then(|b| CompactTarget::from_hex(b).ok())
            .unwrap_or(CompactTarget::from_consensus(REGTEST_DIFFICULTY)),
        nonce: 0, // Set to 0 for now; adjust if mining is needed
    };

    // Create coinbase transaction if not provided
    let height = tmpl
        .as_ref()
        .and_then(|t| t.get("height").and_then(|h| h.as_u64()))
        .unwrap_or(1) as u32;
    let coinbase = coinbase.unwrap_or_else(|| create_coinbase(height));

    // Collect transactions
    let mut transactions = vec![coinbase];
    if let Some(txlist) = txlist {
        transactions.extend(txlist);
    }

    // Create and return the block
    let mut block = Block {
        header,
        txdata: transactions,
    };
    block.header.merkle_root = block.compute_merkle_root().unwrap();

    Ok(block)
}
