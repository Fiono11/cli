use clap::{Parser, Subcommand};
use rand_core::OsRng;
use schnorrkel::{
    olaf::{
        frost::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
        simplpedpop::{AllMessage, SPPOutputMessage},
        SigningKeypair,
    },
    Keypair, PublicKey, SecretKey, Signature,
};
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::SimplpedpopRound1 {
            secret_key,
            recipients,
            output,
        } => {
            let secret_key_string =
                fs::read_to_string(Path::new(&secret_key).join("secret_key.json")).unwrap();
            let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();
            let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes).unwrap());

            let recipients_string =
                fs::read_to_string(Path::new(&recipients).join("recipients.json")).unwrap();
            let recipients_bytes: Vec<Vec<u8>> = from_str(&recipients_string).unwrap();

            let recipients: Vec<PublicKey> = recipients_bytes
                .iter()
                .map(|recipient| PublicKey::from_bytes(recipient).unwrap())
                .collect();

            let all_message: AllMessage =
                keypair.simplpedpop_contribute_all(2, recipients).unwrap();

            let all_message_bytes: Vec<u8> = all_message.to_bytes();
            let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];

            let all_message_json = serde_json::to_string_pretty(&all_message_vec).unwrap();

            let mut all_message_file =
                File::create(Path::new(&output).join("all_messages.json")).unwrap();

            all_message_file
                .write_all(&all_message_json.as_bytes())
                .unwrap();
        }
        Commands::SimplpedpopRound2 {
            secret_key,
            all_messages,
            output,
        } => {
            let secret_key_string =
                fs::read_to_string(Path::new(&secret_key).join("secret_key.json")).unwrap();
            let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();
            let keypair = Keypair::from(SecretKey::from_bytes(&secret_key_bytes).unwrap());

            let all_messages_string =
                fs::read_to_string(Path::new(&all_messages).join("all_messages.json")).unwrap();
            let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string).unwrap();

            let all_messages: Vec<AllMessage> = all_messages_bytes
                .iter()
                .map(|all_message| AllMessage::from_bytes(all_message).unwrap())
                .collect();

            let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages).unwrap();
            let output_round1 = simplpedpop.0;
            let output_json = serde_json::to_string_pretty(&output_round1.to_bytes()).unwrap();

            let mut output_file =
                File::create(Path::new(&output).join("output_round1.json")).unwrap();

            output_file.write_all(&output_json.as_bytes()).unwrap();

            let signing_share = simplpedpop.1;
            let signing_share_json =
                serde_json::to_string_pretty(&signing_share.to_bytes().to_vec()).unwrap();

            let mut signing_share_file =
                File::create(Path::new(&output).join("signing_share.json")).unwrap();

            signing_share_file
                .write_all(&signing_share_json.as_bytes())
                .unwrap();
        }
        Commands::FrostRound1 {
            simplpedpop,
            output,
        } => {
            let signing_share_string =
                fs::read_to_string(Path::new(&simplpedpop).join("signing_share.json")).unwrap();

            let signing_share_bytes: Vec<u8> = from_str(&signing_share_string).unwrap();
            let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

            let (signing_nonces, signing_commitments) = signing_share.commit(&mut OsRng);

            let signing_nonces_json =
                serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec()).unwrap();

            let mut signing_nonces_file =
                File::create(Path::new(&output).join("signing_nonces.json")).unwrap();

            signing_nonces_file
                .write_all(&signing_nonces_json.as_bytes())
                .unwrap();

            let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];

            let signing_commitments_json =
                serde_json::to_string_pretty(&signing_commitments_vec).unwrap();

            let mut signing_commitments_file =
                File::create(Path::new(&output).join("signing_commitments.json")).unwrap();

            signing_commitments_file
                .write_all(&signing_commitments_json.as_bytes())
                .unwrap();
        }
        Commands::FrostRound2 { round1, output } => {
            let signing_commitments_string =
                fs::read_to_string(Path::new(&round1).join("signing_commitments.json")).unwrap();
            let signing_commitments_bytes: Vec<Vec<u8>> =
                from_str(&signing_commitments_string).unwrap();

            let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
                .iter()
                .map(|signing_commitments| {
                    SigningCommitments::from_bytes(signing_commitments).unwrap()
                })
                .collect();

            let signing_nonces_string =
                fs::read_to_string(Path::new(&round1).join("signing_nonces.json")).unwrap();

            let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string).unwrap();
            let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes).unwrap();

            let signing_share_string =
                fs::read_to_string(Path::new(&round1).join("signing_share.json")).unwrap();

            let signing_share_bytes: Vec<u8> = from_str(&signing_share_string).unwrap();
            let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

            let output_string =
                fs::read_to_string(Path::new(&round1).join("output_round1.json")).unwrap();

            let output_bytes: Vec<u8> = from_str(&output_string).unwrap();
            let spp_output = SPPOutputMessage::from_bytes(&output_bytes).unwrap();

            let signing_package = signing_share
                .sign(
                    b"context".to_vec(),
                    b"message".to_vec(),
                    spp_output.spp_output(),
                    signing_commitments,
                    &signing_nonces,
                )
                .unwrap();

            let signing_packages_vec = vec![signing_package.to_bytes()];

            let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec).unwrap();

            let mut signing_package_file =
                File::create(Path::new(&output).join("signing_packages.json")).unwrap();

            signing_package_file
                .write_all(&signing_package_json.as_bytes())
                .unwrap();
        }
        Commands::FrostAggregate {
            signing_packages,
            output,
        } => {
            let signing_packages_string =
                fs::read_to_string(Path::new(&signing_packages).join("signing_packages.json"))
                    .unwrap();
            let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string).unwrap();

            let signing_packages: Vec<SigningPackage> = signing_packages_bytes
                .iter()
                .map(|signing_commitments| SigningPackage::from_bytes(signing_commitments).unwrap())
                .collect();

            let signature: Signature = aggregate(&signing_packages).unwrap();

            let signature_json =
                serde_json::to_string_pretty(&signature.to_bytes().to_vec()).unwrap();

            let mut signature_file =
                File::create(Path::new(&output).join("signature.json")).unwrap();

            signature_file
                .write_all(&signature_json.as_bytes())
                .unwrap();
        }
    }
}

#[derive(Parser)]
#[command(name = "app", about = "An application.", version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SimplpedpopRound1 {
        #[arg(
            long,
            help = "The keypair of the contributor in the SimplPedPoP protocol"
        )]
        secret_key: String,
        #[arg(long, help = "The recipients of the SimplPedPoP protocol")]
        recipients: String,
        #[arg(
            long,
            help = "The output of the first round of the SimplPedPoP protocol"
        )]
        output: String,
    },
    SimplpedpopRound2 {
        #[arg(
            long,
            help = "The keypair of the contributor in the SimplPedPoP protocol"
        )]
        secret_key: String,
        #[arg(
            long,
            help = "The messages from the first rounf of the SimplPedPoP protocol"
        )]
        all_messages: String,
        #[arg(
            long,
            help = "The output of the first round of the SimplPedPoP protocol"
        )]
        output: String,
    },
    FrostRound1 {
        #[arg(long, help = "The output of the SimplPedPoP protocol")]
        simplpedpop: String,
        #[arg(long, help = "The output of the first round of the FROST protocol")]
        output: String,
    },
    FrostRound2 {
        #[arg(long, help = "The output of the round1 the FROST protocol")]
        round1: String,
        #[arg(long, help = "The output of the second round of the FROST protocol")]
        output: String,
    },
    FrostAggregate {
        #[arg(long, help = "The output of the round2 the FROST protocol")]
        signing_packages: String,
        #[arg(long, help = "The final threshold signature the FROST protocol")]
        output: String,
    },
}
