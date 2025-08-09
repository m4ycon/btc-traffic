use std::error::Error;

use bitcoin::Address;
use corepc_node::Client;

#[derive(Debug)]
pub struct AddFundsResult {
    pub address: Address,
    pub amount: f64, // Total funds added in BTC
}

pub async fn add_wallet_funds(
    client: &Client,
    num_blocks: Option<usize>,
) -> Result<AddFundsResult, Box<dyn Error>> {
    let num_blocks = num_blocks.unwrap_or(1);

    // Step 1: Generate a new address in the wallet
    let address = client
        .get_new_address(None, None).unwrap()
        .address().unwrap()
        .assume_checked();

    // Step 2: Mine blocks to the address to add funds (regtest only)
    let _ = client.generate_to_address(num_blocks, &address).unwrap();

    // Step 3: Calculate total funds added (50 BTC per block in regtest, adjusted for halving)
    // Note: Regtest starts with 50 BTC reward; no halving in typical test scenarios
    let amount_per_block = 50.0; // Regtest block reward
    let total_amount = amount_per_block * num_blocks as f64;

    println!("Added {} BTC to address {}", total_amount, address);

    Ok(AddFundsResult {
        address,
        amount: total_amount,
    })
}
