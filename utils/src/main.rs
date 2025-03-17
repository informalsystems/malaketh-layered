use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use genesis::{generate_genesis, make_signers};
use spammer::Spammer;

mod genesis;
mod spammer;
mod tx;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Generate genesis file
    Genesis,

    /// Spam transactions
    #[command(arg_required_else_help = true)]
    Spam(SpamCmd),
}

#[derive(Parser, Debug, Clone, Default, PartialEq)]
pub struct SpamCmd {
    /// Number of transactions to send
    #[clap(short, long, default_value = "0")]
    num_txs: u64,
    #[clap(short, long, default_value = "1000")]
    rate: u64,
    #[clap(short, long, default_value = "0")]
    time: u64,
    /// Spam EIP-4844 (blob) transactions instead of EIP-1559
    #[clap(long, default_value = "false")]
    blobs: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Genesis => generate_genesis(),
        Commands::Spam(SpamCmd {
            num_txs,
            rate,
            time,
            blobs,
        }) => {
            let url = "http://127.0.0.1:8545".parse()?;
            Spammer::new(url, num_txs, time, rate, blobs)?.run().await
        }
    }
}
