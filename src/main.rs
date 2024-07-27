mod balance;
mod circular;
mod claim;
mod consts;
mod cu_limits;
mod easing;
mod logic;
mod miner;
mod price;
mod send_and_confirm;
mod stake;
mod style;
mod utils;
mod views;

use crate::{
    balance::MinerStatus,
    consts::{SOLANA_DEFAULT_KEYPAIR, USER_CONFIG_FILE, WINDOW_SIZE},
    logic::{create_account, FetchMode, Message, ModalType},
    miner::{Config, Miner},
    utils::{get_theme, load_config},
    views::add_account_view,
};
use iced::event::{self};
use iced::{Element, Subscription, Theme};
use price::CoinGecko;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> iced::Result {
    iced::application(Dashboard::title, Dashboard::update, Dashboard::view)
        .theme(Dashboard::theme)
        .window_size(WINDOW_SIZE)
        .exit_on_close_request(false)
        .subscription(Dashboard::subscription)
        .run_with(Dashboard::load)
}
struct Dashboard {
    modal_view: fn(&Dashboard) -> Element<Message>,
    show_modal: ModalType,
    auto_refresh: bool,
    is_refreshed: bool,
    is_claim_process: bool,
    is_stake_process: bool,
    fetch_mode: FetchMode,
    fetch_count: usize,
    data_interval: u64, // Interval for fetching data in seconds
    save_interval: u64, // Interval for saving config in seconds
    is_saved: bool,
    configs: Vec<Config>, // User's config settings
    json_rpc_url: String,
    keypair: String,
    priority_fee: String,
    current_index: Option<usize>, // Current index of selected account
    accounts: Vec<Account>,
    stake: f64,
    stake_usd: String,
    balance: f64,
    balance_usd: String,
    active_num: usize,
    extend_items_per_row: u8, // Extended items per row in UI
    theme: Theme,
    claim_address: String,
    claim_amount: String,
    stake_amount: String,
    version: String,
    dialog: Dialog,
    price_client: Arc<CoinGecko>, // Client for fetching price data
    price_usd: f64,
}

/// Represents a user account with associated data.
#[derive(Clone)]
struct Account {
    json_rpc_url: String,
    miner: Arc<Miner>,
    status: MinerStatus,
    prepared: bool,
}

/// Enum for different content types in dialogs.
#[derive(PartialEq)]
enum ContentType {
    Normal,
    Good,
    Warn,
    Error,
}
impl Default for ContentType {
    fn default() -> Self {
        ContentType::Normal
    }
}

/// Represents the state of a dialog in the UI.
#[derive(Default)]
struct Dialog {
    content: String,
    content_type: ContentType,
}

impl Dashboard {
    fn init() -> Self {
        // Set the default keypair path based on the user's home directory
        let mut default_keypair_path = PathBuf::new();
        if let Some(home_path) = dirs::home_dir() {
            default_keypair_path.push(home_path);
        }
        default_keypair_path.push(SOLANA_DEFAULT_KEYPAIR);

        // Restore account list from user configurations
        let mut accounts: Vec<Account> = vec![];
        let mut user_configs = vec![];
        let mut user_theme = Theme::Light;
        match load_config(USER_CONFIG_FILE) {
            Ok(configs) => {
                // Load user's keypair
                for config in &configs.configs {
                    let account = create_account(
                        config.json_rpc_url.clone(),
                        config.keypair_path.clone(),
                        config.priority_fee,
                    );
                    accounts.push(account);
                }
                user_configs = configs.configs;
                // Load user's preferred theme
                user_theme = get_theme(&configs.theme);
            }
            Err(e) => eprintln!("Failed to load user's config: {}", e),
        }
        println!("Loaded accounts {:?}", accounts.len());

        Self {
            modal_view: add_account_view,
            show_modal: ModalType::Main,
            auto_refresh: true,
            is_refreshed: false,
            is_claim_process: false,
            is_stake_process: false,
            fetch_mode: FetchMode::Parallel,
            fetch_count: 0,
            data_interval: 60,
            save_interval: 5,
            is_saved: true,
            configs: user_configs,
            json_rpc_url: "https://api.devnet.solana.com".to_string(),
            keypair: default_keypair_path.display().to_string(),
            priority_fee: "10".to_string(),
            current_index: None,
            accounts,
            active_num: 0,
            stake: 0.0,
            stake_usd: String::default(),
            balance: 0.0,
            balance_usd: String::default(),
            extend_items_per_row: 0,
            theme: user_theme,
            claim_address: String::default(),
            claim_amount: String::default(),
            stake_amount: String::default(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            dialog: Dialog::default(),
            price_client: Arc::new(CoinGecko::default()),
            price_usd: 0.0,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut events = vec![];

        // Add an event for refreshing data periodically
        if self.auto_refresh {
            events.push(
                iced::time::every(std::time::Duration::from_secs(self.data_interval))
                    .map(|_| Message::Refresh),
            );
        }

        // Add an event for saving user config if not already saved
        if !self.is_saved {
            println!("configs need to save");
            events.push(
                iced::time::every(std::time::Duration::from_secs(self.save_interval))
                    .map(|_| Message::SaveConfig),
            );
        }

        // Listen to general UI events
        events.push(event::listen().map(Message::EventOccurred));

        Subscription::batch(events)
    }

    fn title(&self) -> String {
        String::from("Ore dashboard")
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }
}
