use bitcoin::absolute::Height;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::hashes::Hash;
use bitcoin::opcodes::all::OP_RETURN;
use bitcoin::opcodes::OP_TRUE;
use bitcoin::{Amount, Sequence, Witness, Wtxid};

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
            value: Amount::from_btc(50.).unwrap(),
            script_pubkey: script_pubkey,
        }],
    };

    tx.input[0].witness.push(Wtxid::all_zeros().to_raw_hash());

    tx
}
