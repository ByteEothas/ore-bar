use crate::balance::MinerStatus;
use crate::claim::ClaimParams;
use crate::consts::{
    ACCOUNT_DETAIL_WIDTH, BALANCE_PRECISION, ORE_TOKEN_ID, USD_PRECISION, WINDOW_SIZE,
};
use crate::price::CoinGecko;
use crate::stake::StakeParams;
use crate::utils::{is_valid_path, round_dp, save_config};
use crate::views::dialog_view;
use crate::{
    consts::USER_CONFIG_FILE,
    miner::{Config, Configs, Miner},
    Dashboard,
};
use crate::{Account, ContentType, Dialog};
use iced::event::Event;
use iced::widget::{self};
use iced::{window, Size};
use iced::{Element, Task, Theme};
use rfd::FileDialog;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signer::Signer;
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
    /// Load the initial state of the dashboard.
    pub fn load() -> (Self, Task<Message>) {
        println!("Dashboard load..");
        (
            Dashboard::init(),
            Task::perform(async { Message::Refresh }, |msg| msg),
        )
    }

    /// Update the state of the dashboard based on the received message.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Refresh => {
                println!("Refresh");
                // Disable refresh button
                self.is_refreshed = true;
                match self.fetch_mode {
                    FetchMode::Serial => self.refresh_accounts_serially(),
                    FetchMode::Parallel => self.refresh_accounts_concurrently(),
                }
            }
            Message::AccountsFetched(accounts_status) => {
                #[cfg(debug_assertions)]
                {
                    println!("Data returned by serial");
                }
                // Aggregate account balances and stake amounts
                let (mut total_balance, mut total_stake, mut active_nodes) = (0.0, 0.0, 0);
                for (i, s) in accounts_status.into_iter().enumerate() {
                    if let Some(account) = self.accounts.get_mut(i) {
                        // Update account status
                        account.status = s;
                        account.prepared = true;
                        // Sum account balances
                        total_balance += account.status.balance.parse::<f64>().unwrap_or(0.0);
                        total_stake += account.status.stake.parse::<f64>().unwrap_or(0.0);
                        // Count active nodes
                        active_nodes += if account.status.is_online { 1 } else { 0 };
                    }
                }
                // Summarize accounts' data
                (self.balance, self.stake, self.active_num) =
                    format_accounts_data(total_balance, total_stake, active_nodes);

                // Enable refresh button
                self.is_refreshed = false;
                // Calculate the USD price
                self.calculate_price()
            }
            Message::BalanceFetched(index, status) => {
                #[cfg(debug_assertions)]
                {
                    println!("Data returned by parallel");
                }

                if let Some(account) = self.accounts.get_mut(index) {
                    account.status = status;
                    account.prepared = true;
                }
                // Check if all data has been fetched
                self.fetch_count += 1;
                if self.fetch_count == self.accounts.len() {
                    self.fetch_count = 0;
                    Task::perform(async { Message::Summary }, |msg| msg)
                } else {
                    Task::none()
                }
            }
            Message::Summary => {
                // Summarize accounts' data
                (self.balance, self.stake, self.active_num) = get_accounts_summary(&self.accounts);
                // Enable refresh button
                self.is_refreshed = false;
                // Calculate the USD price
                self.calculate_price()
            }
            Message::PriceFetched(price) => {
                #[cfg(debug_assertions)]
                {
                    println!("Price has been retrieved");
                }
                // Update the USD price display
                if 0.0 == price {
                    self.stake_usd = "--".to_string();
                    self.balance_usd = "--".to_string();
                } else {
                    self.price_usd = price;
                    self.stake_usd = self.get_usd(self.stake).to_string();
                    self.balance_usd = self.get_usd(self.balance).to_string();
                }
                Task::none()
            }
            Message::ToggleSubscription(is_subscribed) => {
                self.auto_refresh = is_subscribed;
                Task::none()
            }
            Message::ToggleFetchMode(is_parallel) => {
                self.fetch_mode = if is_parallel {
                    FetchMode::Parallel
                } else {
                    FetchMode::Serial
                };
                Task::none()
            }
            Message::SetModalView(index, modal_view) => {
                if let Some(index) = index {
                    self.current_index = Some(index);
                }
                self.modal_view = modal_view;
                self.show_modal = ModalType::Sub;
                Task::none()
            }
            // remove this?
            Message::ShowModal(modal_type) => {
                self.show_modal = modal_type;
                widget::focus_next()
            }
            Message::HideModal(message) => {
                let mut commands = vec![Task::perform(
                    async { Message::ShowModal(ModalType::Main) },
                    |msg| msg,
                )];
                if let Some(m) = message {
                    commands.push(Task::perform(async { *m }, |msg| msg))
                }
                Task::batch(commands)
            }
            Message::JsonRpcUrl(url) => {
                self.json_rpc_url = url;
                Task::none()
            }
            Message::Keypair(keypair) => {
                self.keypair = keypair;
                Task::none()
            }
            Message::PriorityFee(fee) => {
                if fee.chars().all(|c| c.is_numeric()) {
                    self.priority_fee = fee;
                }
                Task::none()
            }
            Message::ClaimAddress(address) => {
                self.claim_address = address;
                Task::none()
            }
            Message::ClaimAmount(amount) => {
                self.claim_amount = amount;
                Task::none()
            }
            Message::Claim => {
                // Avoid repeat requests
                self.is_claim_process = true;
                let params = ClaimParams {
                    amount: if let Ok(value) = self.claim_amount.parse::<f64>() {
                        Some(value)
                    } else {
                        None
                    },
                    wallet_address: if !("" == self.claim_address) {
                        Some(self.claim_address.clone())
                    } else {
                        None
                    },
                };
                // Get account from current index
                if let Some(account) = self
                    .accounts
                    .get(self.current_index.expect("No account selected"))
                {
                    // Reset current index
                    self.current_index = None;
                    let miner = Arc::clone(&account.miner);
                    println!("pubkey:{:?}", miner.signer().pubkey());
                    Task::perform(request_claim(miner, params), |msg| {
                        let transaction_status = if msg {
                            TransactionStatus::ClaimSucceed
                        } else {
                            TransactionStatus::ClaimFailed
                        };
                        Message::Callback(transaction_status)
                    })
                } else {
                    Task::none()
                }
            }
            Message::StakeAmount(amount) => {
                self.stake_amount = amount;
                Task::none()
            }
            Message::Stake => {
                // Avoid repeat requests
                self.is_stake_process = true;

                let params = StakeParams {
                    amount: if let Ok(value) = self.stake_amount.parse::<f64>() {
                        Some(value)
                    } else {
                        None
                    },
                    // Currently, only self-staking is allowed
                    sender: None,
                };
                // Get account from current index
                if let Some(account) = self
                    .accounts
                    .get(self.current_index.expect("No account selected"))
                {
                    self.current_index = None;
                    let miner = Arc::clone(&account.miner);
                    Task::perform(request_stake(miner, params), |msg| {
                        let transaction_status = if msg {
                            TransactionStatus::StakeSucceed
                        } else {
                            TransactionStatus::StakeFailed
                        };
                        Message::Callback(transaction_status)
                    })
                } else {
                    Task::none()
                }
            }
            Message::Callback(status) => {
                // Reset stake amount
                self.stake_amount = String::default();
                // Reset claim amount
                self.claim_amount = String::default();
                // Reset stake button
                self.is_stake_process = false;
                // Reset claim button
                self.is_claim_process = false;
                // Set dialog
                self.dialog = match status {
                    TransactionStatus::ClaimSucceed => Dialog {
                        content: "Congratulation! Claim succeeded".to_string(),
                        content_type: ContentType::Good,
                    },
                    TransactionStatus::ClaimFailed => Dialog {
                        content: "Claim failed!".to_string(),
                        content_type: ContentType::Error,
                    },
                    TransactionStatus::StakeSucceed => Dialog {
                        content: "Congratulation! Stake succeeded".to_string(),
                        content_type: ContentType::Good,
                    },
                    TransactionStatus::StakeFailed => Dialog {
                        content: "Stake failed!".to_string(),
                        content_type: ContentType::Error,
                    },
                };
                Task::perform(async { Message::SetModalView(None, dialog_view) }, |msg| {
                    msg
                })
            }
            Message::OpenFile => {
                if let Some(path) = FileDialog::new()
                    .set_title("Open a keypair file...")
                    .pick_file()
                {
                    self.keypair = path.to_str().unwrap_or("").to_string();
                }
                Task::none()
            }
            Message::AddAccount => {
                if !is_valid_path(&self.keypair) {
                    self.dialog = Dialog {
                        content: "No such a keypair file".to_string(),
                        content_type: ContentType::Error,
                    };
                    return Task::perform(
                        async { Message::SetModalView(None, dialog_view) },
                        |msg| msg,
                    );
                }

                let account = create_account(
                    self.json_rpc_url.clone(),
                    self.keypair.clone(),
                    self.priority_fee.parse::<u64>().unwrap_or(10u64),
                );
                self.accounts.push(account);

                // Update user's configs
                let config = Config {
                    json_rpc_url: self.json_rpc_url.clone(),
                    keypair_path: self.keypair.clone(),
                    priority_fee: self.priority_fee.parse::<u64>().unwrap_or(0 as u64),
                };
                self.configs.push(config);
                self.is_saved = false;

                Task::perform(
                    async { Message::HideModal(Some(Box::new(Message::Refresh))) },
                    |msg| msg,
                )
            }
            Message::RemoveAccount(index) => {
                self.current_index = None;
                // Before removing an account, decrease the active_num
                let account = self.accounts.get(index).unwrap();
                if account.status.is_online {
                    self.active_num -= 1;
                }
                // Remove an account
                self.accounts.remove(index);
                // Update user's configs
                self.configs.remove(index);
                self.is_saved = false;

                Task::perform(async { Message::HideModal(None) }, |msg| msg)
            }
            Message::SaveConfig => {
                save_user_config(self);
                Task::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                self.is_saved = false;
                Task::none()
            }
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(window::Event::Resized(Size { width, height: _height })) => {
                        // Calculate the number of items in each row
                        self.extend_items_per_row =
                            ((width as f32 - WINDOW_SIZE.0) / ACCOUNT_DETAIL_WIDTH as f32) as u8;

                        #[cfg(debug_assertions)]
                        {
                            let number_f = (width as f32 - WINDOW_SIZE.0) / ACCOUNT_DETAIL_WIDTH as f32;
                            println!(
                                "width: {:?} height: {:?} items: {:?}/{:?}",
                                width, _height, number_f, self.extend_items_per_row
                            );
                        }
                        return Task::none();
                    }
                    Event::Window(window::Event::CloseRequested) => {
                        if !self.is_saved {
                            save_user_config(self);
                        }
                        return window::get_latest().and_then(window::close);
                    }
                    _ => return Task::none(),
                }
            }
        }
    }

    pub fn refresh_accounts_serially(&self) -> Task<Message> {
        let mut miners = vec![];
        for a in &self.accounts {
            miners.push(Arc::clone(&a.miner));
        }
        Task::perform(fetch_accounts_balance(miners), Message::AccountsFetched)
    }

    pub fn refresh_accounts_concurrently(&self) -> Task<Message> {
        let mut commands = vec![];
        for (i, a) in self.accounts.iter().enumerate() {
            let miner = Arc::clone(&a.miner);
            commands.push(Task::perform(fetch_balance(miner), move |status| {
                Message::BalanceFetched(i, status)
            }));
        }
        Task::batch(commands)
    }

    pub fn get_usd(&self, amount: f64) -> f64 {
        round_dp(amount * self.price_usd, USD_PRECISION)
    }

    pub fn calculate_price(&self) -> Task<Message> {
        let client = Arc::clone(&self.price_client);
        Task::perform(fetch_price(client), Message::PriceFetched)
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
