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
        Commands::GenerateThresholdPublicKeyRound1 { threshold, participant, files } => generate_threshold_public_key_round1(threshold, participant, files).await?,
        Commands::GenerateThresholdPublicKeyRound2 { participant, files } => generate_threshold_public_key_round2(participant, files).await?,
        Commands::ThresholdSignRound1 { participant, files } => threshold_sign_round1(participant, files).await?,
        Commands::ThresholdSignRound2 {
            participant,
            files,
            url,
            pallet,
            call_name,
            call_data,
            context
        } => {
            threshold_sign_round2(participant, files, url, pallet, call_name, call_data, context).await?
        }
        Commands::AggregateThresholdSignature { files } => aggregate_threshold_signature(files).await?,
        Commands::SubmitThresholdExtrinsic { files } => submit_threshold_extrinsic(files).await?,
    }

    Ok(())
}
