use crate::balance::MinerStatus;
use crate::claim::ClaimParams;
use crate::consts::{BALANCE_PRECISION, ORE_TOKEN_ID, USD_PRECISION};
use crate::price::CoinGecko;
use crate::stake::StakeParams;
use crate::utils::{round_dp, save_config};
use crate::Account;
use crate::{
    consts::USER_CONFIG_FILE,
    miner::{Configs, Miner},
    Dashboard,
};
use iced::event::Event;
use iced::{Command, Element, Theme};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    ClaimSucceed,
    ClaimFailed,
    StakeSucceed,
    StakeFailed,
}

#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    AccountsFetched(Vec<MinerStatus>),
    BalanceFetched(usize, MinerStatus),
    PriceFetched(f64),
    Summary,
    ToggleSubscription(bool),
    ToggleFetchMode(bool),
    SetModalView(Option<usize>, fn(&Dashboard) -> Element<Message>),
    ShowModal(ModalType),
    HideModal(Option<Box<Message>>),
    JsonRpcUrl(String),
    Keypair(String),
    PriorityFee(String),
    OpenFile,
    AddAccount,
    RemoveAccount(usize),
    SaveConfig,
    ThemeSelected(Theme),
    ClaimAddress(String),
    ClaimAmount(String),
    Claim,
    StakeAmount(String),
    Stake,
    EventOccurred(Event),
    Callback(TransactionStatus),
}

#[derive(Debug, Clone)]
pub enum ModalType {
    Main,
    Sub,
}

pub enum FetchMode {
    Serial,
    Parallel,
}

impl Dashboard {
    pub fn refresh_accounts_serially(&self) -> Command<Message> {
        let mut miners = vec![];
        for a in &self.accounts {
            miners.push(Arc::clone(&a.miner));
        }
        Command::perform(fetch_accounts_balance(miners), Message::AccountsFetched)
    }

    pub fn refresh_accounts_concurrently(&self) -> Command<Message> {
        let mut commands = vec![];
        for (i, a) in self.accounts.iter().enumerate() {
            let miner = Arc::clone(&a.miner);
            commands.push(Command::perform(fetch_balance(miner), move |status| {
                Message::BalanceFetched(i, status)
            }));
        }
        Command::batch(commands)
    }

    pub fn get_usd(&self, amount: f64) -> f64 {
        round_dp(amount * self.price_usd, USD_PRECISION)
    }

    pub fn calculate_price(&self) -> Command<Message> {
        let client = Arc::clone(&self.price_client);
        Command::perform(fetch_price(client), Message::PriceFetched)
    }
}

pub fn create_account(json_rpc_url: String, keypair_path: String, priority_fee: u64) -> Account {
    let rpc_client =
        RpcClient::new_with_commitment(json_rpc_url.clone(), CommitmentConfig::confirmed());
    let miner = Arc::new(Miner::new(
        Arc::new(rpc_client),
        priority_fee,
        Some(keypair_path.clone()),
    ));
    Account {
        json_rpc_url,
        miner,
        status: MinerStatus::default(),
        prepared: false,
    }
}

pub fn get_accounts_summary(accounts: &Vec<Account>) -> (f64, f64, usize) {
    let mut total_balance = 0 as f64;
    let mut total_stake = 0 as f64;
    let mut active_nodes = 0;
    for a in accounts {
        total_balance += a.status.balance.parse::<f64>().unwrap_or(0.0);
        total_stake += a.status.stake.parse::<f64>().unwrap_or(0.0);
        active_nodes += if a.status.is_online { 1 } else { 0 };
    }
    format_accounts_data(total_balance, total_stake, active_nodes)
}

pub fn format_accounts_data(
    total_balance: f64,
    total_stake: f64,
    active_nodes: usize,
) -> (f64, f64, usize) {
    (
        round_dp(total_balance, BALANCE_PRECISION),
        round_dp(total_stake, BALANCE_PRECISION),
        active_nodes,
    )
}

pub fn save_user_config(dashboard: &mut Dashboard) {
    #[cfg(debug_assertions)]
    {
        println!("{:?},{:?}", dashboard.configs, dashboard.theme);
    }
    match save_config(
        &Configs {
            configs: dashboard.configs.clone(),
            theme: dashboard.theme.to_string(),
        },
        USER_CONFIG_FILE,
    ) {
        Ok(_) => println!("Config saved successfully"),
        Err(e) => eprintln!("Failed to save config: {}", e),
    }
    dashboard.is_saved = true;
}

pub async fn fetch_accounts_balance(miners: Vec<Arc<Miner>>) -> Vec<MinerStatus> {
    let mut accounts_status = vec![];
    for miner in miners {
        // Retrieve the status of an account
        let status = miner.balance(None).await;
        // Gather the status information for an account
        accounts_status.push(status);
    }
    accounts_status
}

pub async fn fetch_balance(miner: Arc<Miner>) -> MinerStatus {
    miner.balance(None).await
}

pub async fn fetch_price(client: Arc<CoinGecko>) -> f64 {
    if let Ok(rep) = client.get().await {
        #[cfg(debug_assertions)]
        {
            println!("Price response: {:?}", rep);
        }
        rep[ORE_TOKEN_ID].usd.unwrap_or_default()
    } else {
        0.0
    }
}

pub async fn request_claim(miner: Arc<Miner>, params: ClaimParams) -> bool {
    miner.claim(params).await
}

pub async fn request_stake(miner: Arc<Miner>, params: StakeParams) -> bool {
    miner.stake(params).await
}
