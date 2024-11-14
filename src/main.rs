mod cli;
mod files;

use crate::cli::commands::{
    multisig::{multisig_aggregate, multisig_round1, multisig_round2},
    simplpedpop::{simplpedpop_round1, simplpedpop_round2},
    submit::submit_transaction,
};
use clap::Parser;
use cli::{Cli, Commands, errors::CliError};

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SimplpedpopRound1 { files } => simplpedpop_round1(files).await?,
        Commands::SimplpedpopRound2 { files } => simplpedpop_round2(files).await?,
        Commands::MultisigRound1 { files } => multisig_round1(files).await?,
        Commands::MultisigRound2 { files } => multisig_round2(files).await?,
        Commands::MultisigAggregate { files } => multisig_aggregate(files).await?,
        Commands::SubmitTransaction {
            files,
            url,
            pallet,
            function,
            call_args,
        } => submit_transaction(files, url, pallet, function, call_args).await?,
    }

    Ok(())
}
