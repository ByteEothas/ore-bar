use crate::consts::{ORE_TOKEN_ID, USD_CURRENCY};
use reqwest::{Client, Error};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

pub struct CoinGecko {
    host: &'static str,
    client: Client,
}

impl Default for CoinGecko {
    fn default() -> Self {
        CoinGecko::new("https://api.coingecko.com/api/v3")
    }
}

impl CoinGecko {
    pub fn new(host: &'static str) -> Self {
        CoinGecko {
            host,
            client: {
                Client::builder()
                    .pool_max_idle_per_host(10) // Set maximum number of idle connections per host
                    .timeout(Duration::from_secs(10)) // Set a request timeout
                    .build()
                    .unwrap()
            },
        }
    }

    pub async fn get(&self) -> Result<HashMap<String, Price>, Error> {
        let req = format!(
            "/simple/price?ids={}&vs_currencies={}",
            ORE_TOKEN_ID, USD_CURRENCY
        );
        self.request(&req).await
    }

    async fn request<R: DeserializeOwned>(&self, endpoint: &str) -> Result<R, Error> {
        self.client
            .get(format!("{host}/{ep}", host = self.host, ep = endpoint))
            .send()
            .await?
            .json()
            .await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Price {
    pub usd: Option<f64>,
}
