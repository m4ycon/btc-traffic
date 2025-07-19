use corepc_node::{Node, P2P};
use std::str::FromStr;
use std::time::Duration;

pub fn node_network() -> Vec<Node> {
    let bitcoind_path = corepc_node::exe_path().expect("Can't find bitcoind executable");

    // todo: set datadir to ramdisk folder for faster nodes.

    // Node 0
    let mut conf_node0 = corepc_node::Conf::default();
    let external = core::net::SocketAddrV4::from_str("127.0.0.1:18444").unwrap();
    conf_node0.p2p = P2P::Connect(external, true);
    let node0 = Node::with_conf(&bitcoind_path, &conf_node0).unwrap();
    node0.client.create_wallet("wallet_0").unwrap();

    vec![node0]
}

#[tokio::main]
async fn main() {
    let nodes = node_network();

    let miner_addr = nodes[0].client.new_address().unwrap();

    loop {
        let res = nodes[0].client.generate_to_address(1, &miner_addr).unwrap();
        println!("{:?}", res);
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
