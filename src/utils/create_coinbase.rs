use bitcoin::absolute::Height;
use bitcoin::blockdata::script::Builder;
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::{Amount, Sequence, Witness};

pub fn create_coinbase(height: u32) -> Transaction {
    // Create coinbase input with height script
    let script_sig = if height <= 16 {
        Builder::new()
            .push_int(height as i64)
            .push_opcode(bitcoin::blockdata::opcodes::OP_TRUE)
            .into_script()
    } else {
        Builder::new().push_int(height as i64).into_script()
    };

    // opreturn, anyone can spend
    let script_pubkey = Builder::new()
        .push_opcode(bitcoin::blockdata::opcodes::OP_TRUE)
        .into_script();

    let mut tx = Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::Blocks(Height::from_consensus(height).unwrap()),
        input: vec![TxIn {
            previous_output: OutPoint::null(),
            script_sig,
            sequence: Sequence::ENABLE_LOCKTIME_NO_RBF,
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: Amount::from_btc(50.).unwrap(), // Initial block reward (adjust for halvings if needed)
            script_pubkey: script_pubkey,
        }],
    };

    // Adjust block reward for halvings (regtest halving every 150 blocks)
    let halvings = height / 150;
    tx.output[0].value = Amount::from_sat(tx.output[0].value.to_sat() >> halvings);

    tx
}
