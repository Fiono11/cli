mod cli;
mod errors;
mod files;
mod multisig;
mod simplpedpop;

use clap::Parser;
use cli::{Cli, Commands};
use errors::CliError;
use multisig::{multisig_aggregate, multisig_round1, multisig_round2};
use simplpedpop::{simplpedpop_round1, simplpedpop_round2};

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SimplpedpopRound1 { files } => simplpedpop_round1(files)?,
        Commands::SimplpedpopRound2 { files } => simplpedpop_round2(files)?,
        Commands::MultisigRound1 { files } => multisig_round1(files)?,
        Commands::MultisigRound2 { files } => multisig_round2(files).await?,
        Commands::MultisigAggregate { files } => multisig_aggregate(files).await?,
    }

    Ok(())
}
