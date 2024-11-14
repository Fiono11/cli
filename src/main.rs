mod cli;
mod files;

use crate::cli::commands::{
    sign::{threshold_sign_round1, threshold_sign_round2},
    generate::{generate_threshold_public_key_round1, generate_threshold_public_key_round2},
    submit::submit_threshold_extrinsic,
};
use clap::Parser;
use cli::{commands::aggregate::aggregate_threshold_signature, errors::CliError, Cli, Commands};

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateThresholdPublicKeyRound1 { files } => generate_threshold_public_key_round1(files).await?,
        Commands::GenerateThresholdPublicKeyRound2 { files } => generate_threshold_public_key_round2(files).await?,
        Commands::ThresholdSignRound1 { files } => threshold_sign_round1(files).await?,
        Commands::ThresholdSignRound2 {
            files,
            url,
            pallet,
            function,
            call_args,
            context
        } => {
            threshold_sign_round2(files, url, pallet, function, call_args, context).await?
        }
        Commands::AggregateThresholdSignatureShares { files } => aggregate_threshold_signature(files).await?,
        Commands::SubmitThresholdTransaction { files } => submit_threshold_extrinsic(files).await?,
    }

    Ok(())
}
