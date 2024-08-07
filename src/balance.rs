use crate::{
    consts::ACTIVE_PERIOD_SECONDS,
    miner::Miner,
    utils::{amount_u64_to_string, get_local_time, try_get_proof},
};
use chrono::{Local, TimeZone};
use ore_api::consts::MINT_ADDRESS;
use solana_program::{native_token::lamports_to_sol, pubkey::Pubkey};
use solana_sdk::signature::Signer;
use std::str::FromStr;

/// Represents the status of a miner, including stake, balance, and activity details.
#[derive(Default, Debug, Clone)]
pub struct MinerStatus {
    /// Indicates if the miner's status is valid.
    pub is_valid: bool,
    /// The quantity of tokens this miner has staked.
    pub stake: String,
    /// The signer authorized to use this proof.
    pub authority: Pubkey,
    /// The quantity of tokens this miner has earned.
    pub balance: String,
    /// The current mining challenge.
    pub challenge: [u8; 32],
    /// The last hash the miner provided.
    pub last_hash: [u8; 32],
    /// The last time this account provided a hash.
    pub last_hash_at: String,
    /// The last time + 60s compare to now
    pub is_online: bool,
    /// The last time stake was deposited into this account.
    pub last_stake_at: String,
    /// The total lifetime hashes provided by this miner.
    pub total_hashes: u64,
    /// The total lifetime rewards distributed to this miner.
    pub total_rewards: u64,
    /// The balance of SOL
    pub sol_balance: f64,
}

impl Miner {
    /// Retrieves the balance and status of the miner associated with the given address.
    /// If no address is provided, the default signer address is used.    
    pub async fn balance(&self, address: Option<String>) -> MinerStatus {
        let signer = self.signer();
        let address = if let Some(address) = address {
            if let Ok(address) = Pubkey::from_str(&address) {
                address
            } else {
                println!("Invalid address: {:?}", address);
                return MinerStatus::default();
            }
        } else {
            signer.pubkey()
        };

        // Try to get the proof associated with the miner's address
        let proof = match try_get_proof(&self.rpc_client, address).await {
            Some(proof) => proof,
            None => {
                let mut status = MinerStatus::default();
                status.authority = address;
                status.is_valid = false;
                return status;
            }
        };

        // Get sol balance
        //if let Ok(Some(balance:u64)) = &self.rpc_client.get_balance(&address).await {balance} else {0.0};
        let sol_balance = lamports_to_sol(
            *&self
                .rpc_client
                .get_balance(&address)
                .await
                .unwrap_or_default(),
        );

        // Get the associated token account balance
        let token_account_address =
            spl_associated_token_account::get_associated_token_address(&address, &MINT_ADDRESS);
        let token_balance = if let Ok(Some(token_account)) = self
            .rpc_client
            .get_token_account(&token_account_address)
            .await
        {
            token_account.token_amount.ui_amount_string
        } else {
            "0".to_string()
        };
        MinerStatus {
            is_valid: true,
            authority: proof.authority,
            balance: token_balance,
            stake: amount_u64_to_string(proof.balance),
            challenge: proof.challenge,
            last_hash: proof.last_hash,
            last_hash_at: get_local_time(proof.last_hash_at),
            is_online: Local::now()
                < Local
                    .timestamp_opt(proof.last_hash_at.saturating_add(ACTIVE_PERIOD_SECONDS), 0)
                    .unwrap(),
            last_stake_at: get_local_time(proof.last_stake_at),
            total_hashes: proof.total_hashes,
            total_rewards: proof.total_rewards,
            sol_balance,
        }
    }
}
