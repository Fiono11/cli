use clap::{Parser, Subcommand};
use schnorrkel::{olaf::simplpedpop::AllMessage, Keypair, PublicKey, SecretKey};
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

            // Convert the Vec<Vec<u8>> to a pretty JSON string
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

            let output_round1 = keypair.simplpedpop_recipient_all(&all_messages).unwrap();
            let output_json = serde_json::to_string_pretty(&output_round1.0.to_bytes()).unwrap();

            let mut output_file =
                File::create(Path::new(&output).join("output_round1.json")).unwrap();

            output_file.write_all(&output_json.as_bytes()).unwrap();

            println!("output: {:?}", output_round1.0.spp_output());
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
}
