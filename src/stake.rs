use crate::{
    cu_limits::CU_LIMIT_CLAIM, send_and_confirm::ComputeBudget, utils::amount_f64_to_u64, Miner,
};
use ore_api::consts::MINT_ADDRESS;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::str::FromStr;

pub struct StakeParams {
    /// The amount of Ore to stake. Defaults to max.
    pub amount: Option<f64>,
    /// Token account to send Ore from.
    pub sender: Option<String>,
}

impl Miner {
    pub async fn stake(&self, params: StakeParams) -> bool {
        let signer = self.signer();
        let sender = match params.sender {
            Some(sender) => Pubkey::from_str(&sender).expect("Failed to parse sender address"),
            None => signer.pubkey(),
        };
        // Get ATA
        let beneficiary =
            spl_associated_token_account::get_associated_token_address(&sender, &MINT_ADDRESS);
        // Get token account
        let Ok(Some(token_account)) = self.rpc_client.get_token_account(&beneficiary).await else {
            println!("Failed to fetch token account");
            return false;
        };
        // Parse amount
        let amount: u64 = if let Some(amount) = params.amount {
            amount_f64_to_u64(amount)
        } else {
            u64::from_str(token_account.token_amount.amount.as_str())
                .expect("Failed to parse token balance")
        };

        // Send tx
        let ix = ore_api::instruction::stake(signer.pubkey(), beneficiary, amount);
        self.send_and_confirm(&[ix], ComputeBudget::Fixed(CU_LIMIT_CLAIM), false)
            .await
            .ok();
        true
    }
}
