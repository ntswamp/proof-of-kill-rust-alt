mod block;
mod blockchain;
mod agent;
mod cli;
mod crypto;
mod fight;

mod server;
mod transaction;
mod utxoset;

//for use of 'info!()'
#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, failure::Error>;

use crate::cli::Cli;
use env_logger::Env;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warning")).init();

    let mut cli = Cli::new();
    if let Err(e) = cli.run() {
        println!("Error: {}", e);
    }
}
