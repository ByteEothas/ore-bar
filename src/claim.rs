use crate::{
    cu_limits::CU_LIMIT_CLAIM,
    miner::Miner,
    send_and_confirm::ComputeBudget,
    utils::{amount_f64_to_u64, get_proof},
};
use ore_api::consts::MINT_ADDRESS;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use std::str::FromStr;

#[derive(Default, Debug, Clone)]
pub struct ClaimParams {
    /// The amount of rewards to claim. Defaults to max.
    pub amount: Option<f64>,
    /// Wallet to receive claimed tokens.
    pub wallet_address: Option<String>,
}

impl Miner {
    pub async fn claim(&self, params: ClaimParams) -> bool {
        let signer = self.signer();
        let pubkey = signer.pubkey();
        let proof = get_proof(&self.rpc_client, pubkey).await;
        let beneficiary = match params.wallet_address {
            Some(wallet_address) => {
                let to_pubkey =
                    Pubkey::from_str(&wallet_address).expect("Failed to parse claim address");
                self.initialize_ata(&to_pubkey).await
            }
            None => self.initialize_ata(&pubkey).await,
        };
        let amount = if let Some(amount) = params.amount {
            amount_f64_to_u64(amount)
        } else {
            proof.balance
        };

        let ix = ore_api::instruction::claim(pubkey, beneficiary, amount);
        self.send_and_confirm(&[ix], ComputeBudget::Fixed(CU_LIMIT_CLAIM), false)
            .await
            .ok();
        true
    }

    async fn initialize_ata(&self, pubkey: &Pubkey) -> Pubkey {
        // Initialize client.
        let client = self.rpc_client.clone();
        // Build instructions.
        let token_account_pubkey =
            spl_associated_token_account::get_associated_token_address(&pubkey, &MINT_ADDRESS);
        // Check if ata already exists
        if let Ok(Some(_ata)) = client.get_token_account(&token_account_pubkey).await {
            return token_account_pubkey;
        }
        // Sign and send transaction.
        let payer = self.signer().pubkey();
        let ix = spl_associated_token_account::instruction::create_associated_token_account(
            &payer,
            &pubkey,
            &MINT_ADDRESS,
            &spl_token::id(),
        );
        self.send_and_confirm(&[ix], ComputeBudget::Dynamic, false)
            .await
            .ok();

        // Return token account address
        token_account_pubkey
    }
}
