# ORE BAR

A tool of [ORE](https://github.com/regolith-labs/ore) designed for easily claiming and staking ORE via a user-friendly graphical interface.

## Features

Ore Bar is developed using [Iced](https://github.com/iced-rs/iced), a cross-platform GUI library for native Rust applications that emphasizes simplicity and type safety.

* Miner status
* Claim
* Stake

## Install

## Run

```sh
ore-bar
```

## Build

```sh
git clone https://github.com/ByteEothas/ore-bar.git
cd ore-bar
cargo run
```

## FAQ

### The system library `glib-2.0` required by crate `glib-sys` was not found.

```sh
sudo apt-get install libgtk-3-dev
```
