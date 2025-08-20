use bitcoin::address::Address;
use bitcoin::{BlockHash};
use clap::Parser;
use corepc_node::{self as node, serde_json, Client};
use node::{Conf, Node, P2P};
use std::net::SocketAddrV4;
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::utils::create_block::create_block;
use crate::utils::create_transaction::create_transaction;
use crate::utils::wallet_funds::add_wallet_funds;

mod utils;

/// Simple regtest traffic generator for bitcoin test and development
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Number of nodes in the network
    #[arg(short = 'n', long, default_value_t = NonZeroU32::new(1).unwrap())]
    nodes: NonZeroU32,

    /// Interval (in seconds) to mine blocks. Will not mine if zero.
    #[arg(short = 't', long, default_value_t = 30)]
    mine_interval: u64,

    /// How many random transactions per block
    #[arg(short = 'x', long, default_value_t = 0)]
    txs_per_block: u32,

    /// External node to connect to (ipv4 + port)
    #[arg(short = 'e', long)]
    external: Option<SocketAddrV4>,

    /// Bitcoin Core executable path
    #[arg(long)]
    bitcoind_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Peer {
    name: String,
    wallet: String,
    node: Node,
    mine_addr: Address,
}

impl Peer {
    fn new(id: &str, external: &Option<P2P>, bitcoind_path: &str) -> Peer {
        let name = String::from_iter(["node_", id]);
        let wallet = String::from_iter(["wallet_", id]);

        let mut conf = Conf::default();
        if let Some(socket) = external {
            conf.p2p = socket.clone();
        }

        let node = Node::with_conf(&bitcoind_path, &conf).unwrap();
        node.client.create_wallet(&wallet).unwrap();
        let mine_addr = node.client.new_address().unwrap();

        Peer {
            name,
            wallet,
            node,
            mine_addr,
        }
    }
}

#[derive(Debug)]
struct Network(Vec<Peer>);

impl Deref for Network {
    type Target = Vec<Peer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Network {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Network {
    fn new(cli: &Cli) -> Network {
        let bitcoind_path = node::exe_path().expect("Can't find bitcoind executable");
        let n = cli.nodes.get();
        let mut network = Vec::with_capacity(n as usize);

        // todo: set datadir to ramdisk folder for faster nodes.

        let mut external = Some(P2P::Connect(
            SocketAddrV4::from_str("127.0.0.1:18444").expect("Can't parse external node address"),
            true,
        ));

        for i in 0..n {
            let peer = Peer::new(&i.to_string(), &external, &bitcoind_path);
            let conn = peer.node.p2p_connect(true).unwrap();
            let _ = external.insert(conn);
            network.push(peer);
        }

        Network(network)
    }

    fn mine(self: &Self, nblocks: Option<usize>) {
        let nblocks = nblocks.unwrap_or(1);
        let size = self.len();
        let n = rand::random_range(0..size);
        println!("Mining with node {}, {} blocks", n, nblocks);

        let addr = &self[n].mine_addr;
        let block = self[n]
            .node
            .client
            .generate_to_address(nblocks, addr)
            .unwrap();
        println!("{:?}", block);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);

    let network = Network::new(&cli);
    println!("{:?}", network);

    let peer = &network[0].node;

    let wallet_funds = add_wallet_funds(&peer.client, None).await.unwrap();

    // network maturity to make above coinbase transaction valid
    // TODO: refactor it to make a global balance so we avoid this solution
    network.mine(Some(100));

    let balances = peer.client.get_balances().unwrap();
    println!("Wallet balances: {:?}", balances);

    let self_transfer = create_transaction(&peer.client, &wallet_funds.address)
        .await
        .unwrap();

    let block = create_block(
        Some(get_prev_hash(&peer.client)),
        None,
        Some(get_min_timestamp(&peer.client)),
        None,
        Some(serde_json::json!({"height":get_block_height(&peer.client) + 1})),
        Some(vec![self_transfer.clone()]),
    )
    .unwrap();
    println!("block {:?}", block);

    peer.client
        .submit_block(&block)
        .map_err(|e| format!("Failed to submit block: {}", e))
        .unwrap();

    network.mine(None);

    let balances = peer.client.get_balances().unwrap();
    println!("Wallet balances: {:?}", balances);

    // find txid of self transfer
    let self_transfer_txid = self_transfer.compute_txid();
    let tx = peer.client.get_transaction(self_transfer_txid).unwrap();
    println!("Self transfer tx: {:#?}", tx);

    loop {
        network.mine(None);
        tokio::time::sleep(Duration::from_secs(cli.mine_interval)).await;
    }
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
