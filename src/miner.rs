use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::{read_keypair_file, Keypair};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub priority_fee: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Configs {
    pub configs: Vec<Config>,
    pub theme: String,
}

pub struct Miner {
    pub keypair_filepath: Option<String>,
    pub priority_fee: u64,
    pub rpc_client: Arc<RpcClient>,
}

impl Miner {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        priority_fee: u64,
        keypair_filepath: Option<String>,
    ) -> Self {
        Self {
            rpc_client,
            keypair_filepath,
            priority_fee,
        }
    }

    pub fn signer(&self) -> Keypair {
        match self.keypair_filepath.clone() {
            Some(filepath) => read_keypair_file(filepath).expect("Failed to read keypair file"),
            None => panic!("No keypair provided"),
        }
    }
}
