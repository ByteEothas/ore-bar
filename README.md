# ORE BAR

A tool of [ORE](https://github.com/regolith-labs/ore) designed for easily claiming and staking ORE via a user-friendly graphical interface.

## Features

Ore Bar is developed using [Iced](https://github.com/iced-rs/iced), a cross-platform GUI library for native Rust applications that emphasizes simplicity and type safety.

* Miner status
* Claim
* Stake

## Install

```sh
cargo install ore-bar@0.0.7-alpha.7
```


## Build

```sh
git clone https://github.com/ByteEothas/ore-bar.git
cd ore-bar
cargo run
```

## Usage

### Import Your Miner's Keypair

To begin, click the Add an Account button on the left panel. Enter your preferred RPC URL, select your keypair file, and specify the gas fee for transactions related to claiming or staking.

### Monitor Your Miner Account Status

You can keep track of each imported account's status, including balance, stake, and the last active time. When a miner account is online, a green indicator appears in the top right corner; if the account is offline, the indicator turns red.

### Claim Your ORE

To claim ORE, click the Claim button on the content panel. Enter the wallet address where you want to receive the ORE. ORE-BAR will automatically convert the wallet address into the associated token address, so you don't need to provide the token address separately. If the inputted address does not have an associated token address, ORE-BAR will create one by initiating a transaction on Solana. By default, if no address is specified, the current account's address is used. Specify the amount of ORE you wish to claim; if left blank, the maximum available amount will be claimed.

### Stake Your ORE

To stake your ORE, click the Stake button on the content panel and enter the amount you wish to stake. Currently, ORE can only be staked to the account from which it originates, so you cannot specify a different wallet address for staking; the staking will be done to your current account.

## FAQ

### The system library `glib-2.0` required by crate `glib-sys` was not found.

```sh
sudo apt-get install libgtk-3-dev
```

### ERROR Insufficient balance

The [ORE](https://github.com/regolith-labs/ore) request requires a minimum balance of 0.005 SOL.