use crate::{
    structs::{
        u64_from_hex_str, Account, BlockHash, LazyBlockHash, Link, Signature, StateBlock,
        StateHashables,
    },
    utils::{get_balance, get_previous, get_rpc_client, handle_receive, handle_send, load_files},
};
use anyhow::Error;
use clap::{Parser, Subcommand};
use ed25519_dalek::VerifyingKey;
use olaf::{
    frost::{aggregate, SigningPackage},
    simplpedpop::{AllMessage, SPPOutputMessage},
    SigningKeypair,
};
use rand_core::OsRng;
use serde_json::from_str;
use std::{
    fmt::{self},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "app", about = "An application.", version = "1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub async fn run(&self) -> Result<(), Error> {
        match &self.command {
            Commands::SimplpedpopRound1 { files } => {
                let file_path: std::path::PathBuf = Path::new(&files).into();

                let secret_key_string =
                    fs::read_to_string(file_path.join("contributor_secret_key.json")).unwrap();

                let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();

                let mut secret_key = [0; 32];
                secret_key.copy_from_slice(&secret_key_bytes);

                let mut keypair = SigningKeypair::from_secret_key(&secret_key);

                let recipients_string =
                    fs::read_to_string(file_path.join("recipients.json")).unwrap();

                let recipients_bytes: Vec<Vec<u8>> = from_str(&recipients_string).unwrap();

                let recipients: Vec<VerifyingKey> = recipients_bytes
                    .iter()
                    .map(|recipient_bytes| {
                        let mut recipient = [0; 32];
                        recipient.copy_from_slice(&recipient_bytes);
                        VerifyingKey::from_bytes(&recipient).unwrap()
                    })
                    .collect();

                let all_message: AllMessage =
                    keypair.simplpedpop_contribute_all(2, recipients).unwrap();

                let all_message_bytes: Vec<u8> = all_message.to_bytes();

                let all_message_vec: Vec<Vec<u8>> = vec![all_message_bytes];

                let all_message_json = serde_json::to_string_pretty(&all_message_vec).unwrap();

                let mut all_message_file =
                    File::create(file_path.join("all_messages.json")).unwrap();

                all_message_file
                    .write_all(&all_message_json.as_bytes())
                    .unwrap();
            }
            Commands::SimplpedpopRound2 { files } => {
                let file_path: std::path::PathBuf = Path::new(&files).into();

                let secret_key_string =
                    fs::read_to_string(file_path.join("recipient_secret_key.json")).unwrap();

                let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();

                let mut secret_key = [0; 32];
                secret_key.copy_from_slice(&secret_key_bytes);

                let mut keypair = SigningKeypair::from_secret_key(&secret_key);

                let all_messages_string =
                    fs::read_to_string(file_path.join("all_messages.json")).unwrap();

                let all_messages_bytes: Vec<Vec<u8>> = from_str(&all_messages_string).unwrap();

                let all_messages: Vec<AllMessage> = all_messages_bytes
                    .iter()
                    .map(|all_message| AllMessage::from_bytes(all_message).unwrap())
                    .collect();

                let simplpedpop = keypair.simplpedpop_recipient_all(&all_messages).unwrap();

                let spp_output = simplpedpop.0.clone();

                let output_json = serde_json::to_string_pretty(&spp_output.to_bytes()).unwrap();

                let mut output_file = File::create(file_path.join("spp_output.json")).unwrap();

                output_file.write_all(&output_json.as_bytes()).unwrap();

                let signing_share = simplpedpop.1;

                let signing_share_json =
                    serde_json::to_string_pretty(&signing_share.to_bytes().to_vec()).unwrap();

                let mut signing_share_file =
                    File::create(file_path.join("signing_share.json")).unwrap();

                signing_share_file
                    .write_all(&signing_share_json.as_bytes())
                    .unwrap();

                let threshold_public_key = simplpedpop.0.spp_output.threshold_public_key;

                let threshold_public_key_json =
                    serde_json::to_string_pretty(&threshold_public_key.0.to_bytes()).unwrap();

                let mut threshold_public_key_file =
                    File::create(file_path.join("threshold_public_key.json")).unwrap();

                threshold_public_key_file
                    .write_all(threshold_public_key_json.as_bytes())
                    .unwrap();

                let threshold_account = Account(threshold_public_key.0.to_bytes());

                println!("{:?}", threshold_account.encode_account());
            }

            Commands::FrostRound1 { files } => {
                let file_path: std::path::PathBuf = Path::new(&files).into();

                let signing_share_string =
                    fs::read_to_string(file_path.join("signing_share.json")).unwrap();

                let signing_share_vec: Vec<u8> = from_str(&signing_share_string).unwrap();

                let mut signing_share_bytes = [0; 64];
                signing_share_bytes.copy_from_slice(&signing_share_vec);

                let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

                let (signing_nonces, signing_commitments) = signing_share.commit(&mut OsRng);

                let signing_nonces_json =
                    serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec()).unwrap();

                let mut signing_nonces_file =
                    File::create(file_path.join("signing_nonces.json")).unwrap();

                signing_nonces_file
                    .write_all(signing_nonces_json.as_bytes())
                    .unwrap();

                let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];

                let signing_commitments_json =
                    serde_json::to_string_pretty(&signing_commitments_vec).unwrap();

                let mut signing_commitments_file =
                    File::create(file_path.join("signing_commitments.json")).unwrap();

                signing_commitments_file
                    .write_all(signing_commitments_json.as_bytes())
                    .unwrap();
            }
            Commands::FrostRound2 {
                action,
                origin,
                amount,
                destination,
                files,
                rpc_url,
            } => {
                let file_path: PathBuf = Path::new(&files).into();
                let (signing_commitments, signing_nonces, signing_share, spp_output_message) =
                    load_files(&file_path)?;

                let threshold_account = Account(
                    spp_output_message
                        .spp_output
                        .threshold_public_key
                        .0
                        .to_bytes(),
                );
                let rpc_client = get_rpc_client(rpc_url.clone()).await;

                let (previous, _) = get_previous(&rpc_client, &origin).await?;

                let balance_rpc = get_balance(&rpc_client, origin).await?;

                let mut balance = balance_rpc;
                let mut link = [0; 32];

                match action {
                    Action::Receive => {
                        let (new_balance, new_link) =
                            handle_receive(&rpc_client, &origin, balance).await?;
                        balance = new_balance;
                        link = new_link;
                    }
                    Action::Send => {
                        let amount = amount.expect("A send amount is needed!");
                        let destination = destination
                            .as_ref()
                            .expect("A destination account is needed!");
                        let (new_balance, new_link) =
                            handle_send(&rpc_client, &destination, amount, balance).await?;
                        balance = new_balance;
                        link = new_link;
                    }
                    _ => {}
                }

                let hashables = StateHashables {
                    previous: BlockHash(previous),
                    balance,
                    link: Link(link),
                    representative: threshold_account,
                    account: threshold_account,
                };

                let hash = LazyBlockHash::new();
                let tx_hash = hash.hash(&hashables).0;

                let signing_package = signing_share
                    .sign(
                        &tx_hash,
                        &spp_output_message.spp_output,
                        &signing_commitments,
                        &signing_nonces,
                    )
                    .unwrap();

                let signing_packages_vec = vec![signing_package.to_bytes()];

                let signing_package_json =
                    serde_json::to_string_pretty(&signing_packages_vec).unwrap();
                let mut signing_package_file =
                    fs::File::create(file_path.join("signing_packages.json"))?;
                signing_package_file.write_all(signing_package_json.as_bytes())?;
            }
            Commands::FrostAggregate {
                action,
                origin,
                amount,
                destination,
                files,
                rpc_url,
            } => {
                let file_path: PathBuf = Path::new(&files).into();

                let signing_packages_string =
                    fs::read_to_string(file_path.join("signing_packages.json"))?;
                let signing_packages_bytes: Vec<Vec<u8>> =
                    serde_json::from_str(&signing_packages_string)?;
                let signing_packages: Vec<SigningPackage> = signing_packages_bytes
                    .iter()
                    .map(|bytes| SigningPackage::from_bytes(bytes).unwrap())
                    .collect();

                let signature = Signature {
                    bytes: aggregate(&signing_packages).unwrap().to_bytes(),
                };

                let output_string = fs::read_to_string(file_path.join("spp_output.json"))?;
                let output_bytes: Vec<u8> = serde_json::from_str(&output_string)?;
                let spp_output_message = SPPOutputMessage::from_bytes(&output_bytes).unwrap();
                let threshold_account = Account(
                    spp_output_message
                        .spp_output
                        .threshold_public_key
                        .0
                        .to_bytes(),
                );

                let rpc_client = get_rpc_client(rpc_url.clone()).await;

                let (previous, previous_str) = get_previous(&rpc_client, &origin).await?;

                let balance_rpc = get_balance(&rpc_client, origin).await?;

                let mut balance = balance_rpc;
                let mut link = [0; 32];

                match action {
                    Action::Receive => {
                        let (new_balance, new_link) =
                            handle_receive(&rpc_client, &origin, balance).await?;
                        balance = new_balance;
                        link = new_link;
                    }
                    Action::Send => {
                        let amount = amount.expect("A send amount is needed!");
                        let destination = destination
                            .as_ref()
                            .expect("A destination account is needed!");
                        let (new_balance, new_link) =
                            handle_send(&rpc_client, &destination, amount, balance).await?;
                        balance = new_balance;
                        link = new_link;
                    }
                    _ => {}
                }

                let hashables = StateHashables {
                    previous: BlockHash(previous),
                    balance,
                    link: Link(link),
                    representative: threshold_account,
                    account: threshold_account,
                };

                let hash = LazyBlockHash::new();
                let response = rpc_client.work_generate(&previous_str).await?;
                let work_str = response.get("work").unwrap().as_str().unwrap();
                let work = u64_from_hex_str(work_str)?;

                let block = StateBlock {
                    work,
                    signature,
                    hashables,
                    hash,
                    sideband: None,
                };

                rpc_client
                    .process(
                        &action.to_string(),
                        &serde_json::to_string_pretty(&block).unwrap(),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}

#[derive(Subcommand)]
pub enum Commands {
    SimplpedpopRound1 {
        #[arg(long)]
        files: String,
    },
    SimplpedpopRound2 {
        #[arg(long)]
        files: String,
    },
    FrostRound1 {
        #[arg(long)]
        files: String,
    },
    FrostRound2 {
        #[arg(long)]
        action: Action,
        #[arg(long)]
        origin: String,
        #[arg(long)]
        amount: Option<u128>,
        #[arg(long)]
        destination: Option<String>,
        #[arg(long)]
        files: String,
        #[arg(long)]
        rpc_url: Option<String>,
    },
    FrostAggregate {
        #[arg(long)]
        action: Action,
        #[arg(long)]
        origin: String,
        #[arg(long)]
        amount: Option<u128>,
        #[arg(long)]
        destination: Option<String>,
        #[arg(long)]
        files: String,
        #[arg(long)]
        rpc_url: Option<String>,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Action {
    Send,
    Receive,
    Open,
    Change,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let action_str = match self {
            Action::Send => "send",
            Action::Receive => "receive",
            Action::Open => "open",
            Action::Change => "change",
        };
        write!(f, "{}", action_str)
    }
}
