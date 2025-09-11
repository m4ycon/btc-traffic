use bitcoin::absolute::Height;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::hashes::Hash;
use bitcoin::opcodes::all::OP_RETURN;
use bitcoin::opcodes::OP_TRUE;
use bitcoin::{Amount, Sequence, Witness, Wtxid};

const INITIAL_SUBSIDY: u64 = 50_0000_0000; // 50 BTC in satoshis
const HALVING_INTERVAL: u32 = 210_000;

pub fn calculate_subsidy(height: u32) -> Amount {
    let halvings = height / HALVING_INTERVAL;
    let subsidy_sat = INITIAL_SUBSIDY >> halvings;
    Amount::from_sat(subsidy_sat)
}

pub fn create_coinbase(height: u32) -> Transaction {
    // Create coinbase input with height script
    let script_sig = if height <= 16 {
        Builder::new()
            .push_int(height as i64)
            .push_opcode(OP_TRUE)
            .into_script()
    } else {
        Builder::new().push_int(height as i64).into_script()
    };

    // anyone can spend
    let script_pubkey = Builder::new()
        .push_opcode(OP_RETURN)
        .into_script();

    let value = calculate_subsidy(height);

    let mut tx = Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::Blocks(Height::from_consensus(height).unwrap()),
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig,
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value,
            script_pubkey,
        }],
    };

    tx.input[0].witness.push(Wtxid::all_zeros().to_raw_hash());

    tx
}
