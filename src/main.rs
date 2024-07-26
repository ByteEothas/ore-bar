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
mod utils;
mod views;

use crate::claim::ClaimParams;
use crate::stake::StakeParams;
use crate::{
    balance::MinerStatus,
    consts::{
        ACCOUNT_DETAIL_WIDTH, MENU_CATEGORY_SPACING, MENU_ITEM_INDENT, MENU_ITEM_SPACING,
        MENU_SPAN_HEIGHT, SOLANA_DEFAULT_KEYPAIR, SUBHEAD_TEXT, USER_CONFIG_FILE, WINDOW_SIZE,
    },
    logic::{
        create_account, format_accounts_data, get_accounts_summary, request_claim, request_stake,
        save_user_config, FetchMode, Message, ModalType, TransactionStatus,
    },
    miner::{Config, Miner},
    utils::{abbreviate, get_theme, is_valid_path, load_config},
    views::modal::Modal,
    views::{active_num_view, add_account_view, dialog_view, get_content_list, get_svg_icon},
};
use iced::event::{self, Event};
use iced::executor;
use iced::widget::{self};
use iced::widget::{button, checkbox, column, pick_list, row, text, vertical_space};
use iced::{window, Application, Command, Element, Length, Padding, Settings, Subscription, Theme};
use ore_api::consts::MINT_ADDRESS;
use price::CoinGecko;
use rfd::FileDialog;
use serde::Deserialize;
use solana_sdk::signer::Signer;
use std::fs::{self};
use std::path::PathBuf;
use std::sync::Arc;
use toml;

fn main() -> iced::Result {
    Dashboard::run(Settings::default())
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    version: String,
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

impl Application for Dashboard {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let cargo_toml_content =
            fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
        let cargo_toml: CargoToml =
            toml::from_str(&cargo_toml_content).expect("Failed to parse Cargo.toml");

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

        (
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
                version: cargo_toml.package.version,
                dialog: Dialog::default(),
                price_client: Arc::new(CoinGecko::default()),
                price_usd: 0.0,
            },
            Command::perform(async { Message::Refresh }, |msg| msg),
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
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
                    Command::perform(async { Message::Summary }, |msg| msg)
                } else {
                    Command::none()
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
                Command::none()
            }
            Message::ToggleSubscription(is_subscribed) => {
                self.auto_refresh = is_subscribed;
                Command::none()
            }
            Message::ToggleFetchMode(is_parallel) => {
                self.fetch_mode = if is_parallel {
                    FetchMode::Parallel
                } else {
                    FetchMode::Serial
                };
                Command::none()
            }
            Message::SetModalView(index, modal_view) => {
                if let Some(index) = index {
                    self.current_index = Some(index);
                }
                self.modal_view = modal_view;
                self.show_modal = ModalType::Sub;
                Command::none()
            }
            // remove this?
            Message::ShowModal(modal_type) => {
                self.show_modal = modal_type;
                widget::focus_next()
            }
            Message::HideModal(message) => {
                let mut commands = vec![Command::perform(
                    async { Message::ShowModal(ModalType::Main) },
                    |msg| msg,
                )];
                if let Some(m) = message {
                    commands.push(Command::perform(async { *m }, |msg| msg))
                }
                Command::batch(commands)
            }
            Message::JsonRpcUrl(url) => {
                self.json_rpc_url = url;
                Command::none()
            }
            Message::Keypair(keypair) => {
                self.keypair = keypair;
                Command::none()
            }
            Message::PriorityFee(fee) => {
                if fee.chars().all(|c| c.is_numeric()) {
                    self.priority_fee = fee;
                }
                Command::none()
            }
            Message::ClaimAddress(address) => {
                self.claim_address = address;
                Command::none()
            }
            Message::ClaimAmount(amount) => {
                self.claim_amount = amount;
                Command::none()
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
                    Command::perform(request_claim(miner, params), |msg| {
                        let transaction_status = if msg {
                            TransactionStatus::ClaimSucceed
                        } else {
                            TransactionStatus::ClaimFailed
                        };
                        Message::Callback(transaction_status)
                    })
                } else {
                    Command::none()
                }
            }
            Message::StakeAmount(amount) => {
                self.stake_amount = amount;
                Command::none()
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
                    Command::perform(request_stake(miner, params), |msg| {
                        let transaction_status = if msg {
                            TransactionStatus::StakeSucceed
                        } else {
                            TransactionStatus::StakeFailed
                        };
                        Message::Callback(transaction_status)
                    })
                } else {
                    Command::none()
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
                Command::perform(async { Message::SetModalView(None, dialog_view) }, |msg| {
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
                Command::none()
            }
            Message::AddAccount => {
                if !is_valid_path(&self.keypair) {
                    self.dialog = Dialog {
                        content: "No such a keypair file".to_string(),
                        content_type: ContentType::Error,
                    };
                    return Command::perform(
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

                Command::perform(
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

                Command::perform(async { Message::HideModal(None) }, |msg| msg)
            }
            Message::SaveConfig => {
                save_user_config(self);
                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;
                self.is_saved = false;
                Command::none()
            }
            Message::EventOccurred(event) => {
                match event {
                    Event::Window(
                        _id,
                        window::Event::Resized {
                            width,
                            height: _height,
                        },
                    ) => {
                        // Calculate the number of items in each row
                        self.extend_items_per_row =
                            ((width as f32 - WINDOW_SIZE.0) / ACCOUNT_DETAIL_WIDTH as f32) as u8;

                        #[cfg(debug_assertions)]
                        {
                            let number_f =
                                (width as f32 - WINDOW_SIZE.0) / ACCOUNT_DETAIL_WIDTH as f32;
                            println!(
                                "width: {:?} height: {:?} items: {:?}/{:?}",
                                width, _height, number_f, self.extend_items_per_row
                            );
                        }
                        return Command::none();
                    }
                    Event::Window(id, window::Event::CloseRequested) => {
                        if !self.is_saved {
                            save_user_config(self);
                        }
                        return window::close(id);
                    }
                    _ => return Command::none(),
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let refresh_button = if !self.is_refreshed {
            button(get_svg_icon("refresh", 24, 24)).on_press(Message::Refresh)
        } else {
            button(get_svg_icon("refresh-disabled", 24, 24))
        };

        let left = column![
            row![text("Accounts").size(20), refresh_button].align_items(iced::Alignment::Center),
            row![
                column![
                    text("Number:"),
                    text("Balance:").height(MENU_SPAN_HEIGHT),
                    text("Stake:").height(MENU_SPAN_HEIGHT),
                    text("Status:"),
                    text("Mint Address:")
                ]
                .padding(Padding::from([0, 0, 0, MENU_ITEM_INDENT]))
                .spacing(MENU_ITEM_SPACING),
                column![
                    text(self.accounts.len()),
                    column![
                        text(&self.balance),
                        text(format!("${}", &self.balance_usd)).size(SUBHEAD_TEXT)
                    ]
                    .height(MENU_SPAN_HEIGHT)
                    .width(Length::Fill)
                    .align_items(iced::Alignment::End),
                    column![
                        text(&self.stake),
                        text(format!("${}", &self.stake_usd)).size(SUBHEAD_TEXT)
                    ]
                    .height(MENU_SPAN_HEIGHT)
                    .width(Length::Fill)
                    .align_items(iced::Alignment::End),
                    active_num_view(&self),
                    text(abbreviate(&MINT_ADDRESS.to_string()))
                ]
                .align_items(iced::Alignment::End)
                .spacing(MENU_ITEM_SPACING)
            ],
            column![
                checkbox("Auto Refresh", self.auto_refresh).on_toggle(Message::ToggleSubscription),
                checkbox(
                    "Parallel Mode",
                    match self.fetch_mode {
                        FetchMode::Parallel => true,
                        FetchMode::Serial => false,
                    }
                )
                .on_toggle(Message::ToggleFetchMode),
            ]
            .spacing(MENU_ITEM_SPACING),
            row![button(
                text("Add an account").horizontal_alignment(iced::alignment::Horizontal::Center)
            )
            .on_press(Message::SetModalView(None, add_account_view))
            .width(Length::Fill)],
            vertical_space(),
            // Themes
            row!(pick_list(
                Theme::ALL,
                Some(&self.theme),
                Message::ThemeSelected
            ))
            .width(100),
            row![text("Version:"), text(&self.version)],
        ]
        .spacing(MENU_CATEGORY_SPACING)
        .padding(Padding::from([5, 5, 5, 10]))
        .align_items(iced::Alignment::Start);
        let content = get_content_list(self);
        let body = row![left.width(250), content];
        match &self.show_modal {
            ModalType::Sub => Modal::new(body, (self.modal_view)(&self)).into(),
            _ => body.into(),
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
