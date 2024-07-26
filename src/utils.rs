use crate::consts::SHOW_RPC_URL_MAX_LENGTH;
use crate::miner::{Config, Configs};
use cached::proc_macro::cached;
use chrono::{Local, TimeZone};
use iced::Theme;
use ore_api::{
    self,
    consts::{MINT_ADDRESS, PROOF, TOKEN_DECIMALS, TREASURY_ADDRESS},
    state::Proof,
};
use ore_utils::AccountDeserialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use url::Url;

pub async fn get_proof(client: &RpcClient, authority: Pubkey) -> Proof {
    let proof_address = proof_pubkey(authority);
    let data = client
        .get_account_data(&proof_address)
        .await
        .expect("Failed to get miner account");
    *Proof::try_from_bytes(&data).expect("Failed to parse miner account")
}

pub async fn try_get_proof(client: &RpcClient, authority: Pubkey) -> Option<Proof> {
    let proof_address = proof_pubkey(authority);
    match client.get_account_data(&proof_address).await {
        Ok(data) => Some(*Proof::try_from_bytes(&data).expect("Failed to parse miner account")),
        Err(_) => None,
    }
}

pub fn amount_u64_to_string(amount: u64) -> String {
    amount_u64_to_f64(amount).to_string()
}

pub fn amount_u64_to_f64(amount: u64) -> f64 {
    (amount as f64) / 10f64.powf(TOKEN_DECIMALS as f64)
}

pub fn amount_f64_to_u64(amount: f64) -> u64 {
    (amount * 10f64.powf(TOKEN_DECIMALS as f64)) as u64
}

#[cached]
pub fn proof_pubkey(authority: Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &ore_api::ID).0
}

#[cached]
pub fn treasury_tokens_pubkey() -> Pubkey {
    get_associated_token_address(&TREASURY_ADDRESS, &MINT_ADDRESS)
}

pub fn abbreviate(s: &str) -> String {
    // Define the number of characters to keep from the start and end
    let keep = 4;
    if s.len() <= 2 * keep {
        return s.to_string();
    }
    let start = &s[..keep];
    let end = &s[s.len() - keep..];
    format!("{}...{}", start, end)
}

pub fn get_domain(url: &str) -> String {
    let Ok(parse_url) = Url::parse(url) else {
        return String::default();
    };
    fixed_string(parse_url.host_str().unwrap(), SHOW_RPC_URL_MAX_LENGTH)
}

pub fn fixed_string(s: &str, max_length: usize) -> String {
    let s_length = s.len();
    s[..if max_length < s_length {
        max_length
    } else {
        s_length
    }]
        .to_string()
}

pub fn get_local_time(timestamp: i64) -> String {
    let local = Local.timestamp_opt(timestamp, 0).unwrap();
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn save_config(configs: &Configs, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string(configs)?;
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

pub fn load_config(file_path: &str) -> Result<Configs, Box<dyn std::error::Error>> {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return Ok(Configs::default()),
    };
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let configs: Configs = toml::from_str(&content)?;
    Ok(configs)
}

pub fn append_config(config: Config, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut configs = load_config(file_path)?;
    configs.configs.push(config);

    let toml_string = toml::to_string(&configs)?;
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

pub fn is_valid_path(file_path: &str) -> bool {
    let path = Path::new(file_path);
    if path.exists() {
        if let Ok(metadata) = fs::metadata(path) {
            return metadata.is_file();
        }
    }
    false
}

pub fn get_theme(name: &str) -> Theme {
    match name {
        "Light" => Theme::Light,
        "Dark" => Theme::Dark,
        "Dracula" => Theme::Dracula,
        "Nord" => Theme::Nord,
        "Solarized Light" => Theme::SolarizedLight,
        "Solarized Dark" => Theme::SolarizedDark,
        "Gruvbox Light" => Theme::GruvboxLight,
        "Gruvbox Dark" => Theme::GruvboxDark,
        "Catppuccin Latte" => Theme::CatppuccinLatte,
        "Catppuccin Frappé" => Theme::CatppuccinFrappe,
        "Catppuccin Macchiato" => Theme::CatppuccinMacchiato,
        "Catppuccin Mocha" => Theme::CatppuccinMocha,
        "Tokyo Night" => Theme::TokyoNight,
        "Tokyo Night Storm" => Theme::TokyoNightStorm,
        "Tokyo Night Light" => Theme::TokyoNightLight,
        "Kanagawa Wave" => Theme::KanagawaWave,
        "Kanagawa Dragon" => Theme::KanagawaDragon,
        "Kanagawa Lotus" => Theme::KanagawaLotus,
        "Moonfly" => Theme::Moonfly,
        "Nightfly" => Theme::Nightfly,
        "Oxocarbon" => Theme::Oxocarbon,
        "Ferra" => Theme::Ferra,
        _ => Theme::Light,
    }
}

pub fn round_dp(decimal: f64, precision: u8) -> f64 {
    let factor = 10f64.powi(precision.into());
    (decimal * factor).round() / factor
}
