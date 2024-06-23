use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    str::FromStr,
};

use anyhow::Error;
use clap::{Parser, Subcommand};
use ed25519_dalek::VerifyingKey;
use olaf::{
    frost::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
    simplpedpop::{AllMessage, SPPOutputMessage},
    SigningKeypair,
};
use rand_core::OsRng;
use reqwest::Url;
use serde_json::from_str;

use crate::{
    rpc::RpcClient,
    structs::{
        u64_from_hex_str, Account, Amount, BlockHash, LazyBlockHash, Link, Signature, StateBlock,
        StateHashables,
    },
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
                amount,
                destination,
                files,
                rpc_url,
            } => {
                let file_path: std::path::PathBuf = Path::new(&files).into();

                let signing_commitments_string =
                    fs::read_to_string(Path::new(&files).join("signing_commitments.json")).unwrap();

                let signing_commitments_bytes: Vec<Vec<u8>> =
                    from_str(&signing_commitments_string).unwrap();

                let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
                    .iter()
                    .map(|signing_commitments| {
                        SigningCommitments::from_bytes(signing_commitments).unwrap()
                    })
                    .collect();

                let signing_nonces_string =
                    fs::read_to_string(file_path.join("signing_nonces.json")).unwrap();

                let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string).unwrap();

                let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes).unwrap();

                let signing_share_string =
                    fs::read_to_string(file_path.join("signing_share.json")).unwrap();

                let signing_share_vec: Vec<u8> = from_str(&signing_share_string).unwrap();

                let mut signing_share_bytes = [0; 64];
                signing_share_bytes.copy_from_slice(&signing_share_vec);

                let signing_share = SigningKeypair::from_bytes(&signing_share_bytes).unwrap();

                let output_string = fs::read_to_string(file_path.join("spp_output.json")).unwrap();

                let output_bytes: Vec<u8> = from_str(&output_string).unwrap();
                let spp_output_message = SPPOutputMessage::from_bytes(&output_bytes).unwrap();
                let spp_output = spp_output_message.spp_output;

                //let source_string = fs::read_to_string(file_path.join("source.json")).unwrap();

                let hex =
                    hex::decode("00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C")
                        .unwrap();

                //let source_bytes: Vec<u8> = from_str(&source_string).unwrap();
                let mut bytes = [0; 32];
                bytes.copy_from_slice(&hex);

                let threshold_account = Account(spp_output.threshold_public_key.0.to_bytes());

                println!("{:?}", threshold_account.encode_account());

                let source = BlockHash(bytes);

                //let previous_str = "0000000000000000000000000000000000000000000000000000000000000000";
                /*let previous_str = "5B34BD0C75449552CA02DA99E01A03DB6E9740EA345F8BE1260DD12026B2225B";
                let previous_decoded = hex::decode(previous_str).unwrap();
                let mut previous = [0; 32];
                previous.copy_from_slice(&previous_decoded);*/

                let rpc_client = if let Some(url) = rpc_url {
                    RpcClient::new(Url::from_str(url).unwrap())
                } else {
                    RpcClient::new(Url::from_str("https://rpcproxy.bnano.info/proxy").unwrap())
                };

                let account_history: serde_json::Value = rpc_client
                    .account_history(
                        "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                        1,
                    )
                    .await
                    .unwrap();

                println!("account history: {:?}", account_history);

                let previous_str = account_history
                    .get("history")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .first()
                    .unwrap()
                    .get("hash")
                    .unwrap()
                    .as_str()
                    .unwrap();

                println!("str: {:?}", previous_str);

                let previous_decoded = hex::decode(previous_str).unwrap();
                let mut previous = [0; 32];
                previous.copy_from_slice(&previous_decoded);

                //let link_str = "00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C";
                //let link_decoded = hex::decode(link_str).unwrap();

                let link_nano = "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q";
                let link_decoded = Account::decode_account(link_nano).unwrap();

                let account_balance = rpc_client
                    .account_balance(
                        "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                    )
                    .await
                    .unwrap();

                let balance_rpc = account_balance.get("balance").unwrap();

                let mut balance = Amount::raw(0);

                let mut link = [0; 32];

                if action == "receive" {
                    let receivable = rpc_client
                        .receivable(
                            "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                            1,
                        )
                        .await
                        .unwrap();

                    println!("{:?}", receivable);

                    let blocks = receivable
                        .get("blocks")
                        .ok_or_else(|| anyhow::anyhow!("Missing blocks field"))?
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("Blocks field is not an object"))?;

                    let (key, value) = blocks.iter().next().unwrap();

                    let link_decoded = hex::decode(key).unwrap();
                    link.copy_from_slice(&link_decoded);

                    balance = Amount::raw(str::parse(&value.as_str().unwrap()).unwrap())
                        + Amount::raw(str::parse(balance_rpc.as_str().unwrap()).unwrap())
                } else {
                    let link_decoded =
                        Account::decode_account(destination.as_ref().unwrap()).unwrap();
                    link.copy_from_slice(&link_decoded.0);

                    balance = Amount::raw(str::parse(balance_rpc.as_str().unwrap()).unwrap())
                        - Amount::raw(amount.unwrap())
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

                println!("hash: {:?}", tx_hash);

                let signing_package = signing_share
                    .sign(&tx_hash, &spp_output, &signing_commitments, &signing_nonces)
                    .unwrap();

                let signing_packages_vec = vec![signing_package.to_bytes()];

                let signing_package_json =
                    serde_json::to_string_pretty(&signing_packages_vec).unwrap();

                let mut signing_package_file =
                    File::create(file_path.join("signing_packages.json")).unwrap();

                signing_package_file
                    .write_all(signing_package_json.as_bytes())
                    .unwrap();
            }
            Commands::FrostAggregate {
                action,
                amount,
                destination,
                files,
                rpc_url,
            } => {
                let file_path: std::path::PathBuf = Path::new(&files).into();

                let signing_packages_string =
                    fs::read_to_string(file_path.join("signing_packages.json")).unwrap();

                let signing_packages_bytes: Vec<Vec<u8>> =
                    from_str(&signing_packages_string).unwrap();

                let signing_packages: Vec<SigningPackage> = signing_packages_bytes
                    .iter()
                    .map(|signing_commitments| {
                        SigningPackage::from_bytes(signing_commitments).unwrap()
                    })
                    .collect();

                let signature = Signature {
                    bytes: aggregate(&signing_packages).unwrap().to_bytes(),
                };

                let hex_sig = hex::encode(signature.bytes);

                println!("sig: {:?}", hex_sig);

                // Hexadecimal string
                //let hex_str = "de7c32d590b43708";
                let hex_str = "181207ac52a4ac83";

                // Decode the hexadecimal string to a byte array
                let bytes = hex::decode(hex_str).unwrap();

                // Ensure the byte array is exactly 8 bytes long
                let bytes_array: [u8; 8] = bytes.try_into().expect("slice with incorrect length");

                // Convert the byte array to a u64
                let work = u64_from_hex_str(hex_str).unwrap();

                let hex =
                    hex::decode("00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C")
                        .unwrap();

                //let source_bytes: Vec<u8> = from_str(&source_string).unwrap();
                let mut bytes = [0; 32];
                bytes.copy_from_slice(&hex);

                let output_string = fs::read_to_string(file_path.join("spp_output.json")).unwrap();

                let output_bytes: Vec<u8> = from_str(&output_string).unwrap();
                let spp_output_message = SPPOutputMessage::from_bytes(&output_bytes).unwrap();
                let spp_output = spp_output_message.spp_output;

                let threshold_account = Account(spp_output.threshold_public_key.0.to_bytes());

                println!("{:?}", threshold_account.encode_account());

                let source = BlockHash(bytes);

                //let previous_str = "0000000000000000000000000000000000000000000000000000000000000000";
                let previous_str =
                    "5B34BD0C75449552CA02DA99E01A03DB6E9740EA345F8BE1260DD12026B2225B";
                let previous_decoded = hex::decode(previous_str).unwrap();
                let mut previous = [0; 32];
                previous.copy_from_slice(&previous_decoded);

                let link_nano = "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q";

                let link_str = "00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C";
                //let link_decoded = hex::decode(link_str).unwrap();
                let link_decoded = Account::decode_account(link_nano).unwrap();
                //let mut link = [0; 32];
                //link.copy_from_slice(&link_decoded.0);

                let rpc_client = if let Some(url) = rpc_url {
                    RpcClient::new(Url::from_str(url).unwrap())
                } else {
                    RpcClient::new(Url::from_str("https://rpcproxy.bnano.info/proxy").unwrap())
                };

                let account_history: serde_json::Value = rpc_client
                    .account_history(
                        "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                        1,
                    )
                    .await
                    .unwrap();

                println!("account history: {:?}", account_history);

                let previous_str = account_history
                    .get("history")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .first()
                    .unwrap()
                    .get("hash")
                    .unwrap()
                    .as_str()
                    .unwrap();

                let previous_decoded = hex::decode(previous_str).unwrap();
                let mut previous = [0; 32];
                previous.copy_from_slice(&previous_decoded);

                let receivable = rpc_client
                    .receivable(
                        "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                        1,
                    )
                    .await
                    .unwrap();

                /*let link_str = receivable
                    .get("blocks")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .get(0)
                    .unwrap()
                    .as_str()
                    .unwrap();

                let link_decoded = hex::decode(link_str).unwrap();
                let mut link = [0; 32];
                link.copy_from_slice(&link_decoded);*/

                //let link_decoded = hex::decode(key).unwrap();
                //let mut link = [0; 32];
                //link.copy_from_slice(&link_decoded);

                //println!(
                //"{:?}",
                //Amount::decode_hex(value.as_str().unwrap()).unwrap().raw
                //);

                let account_balance = rpc_client
                    .account_balance(
                        "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                    )
                    .await
                    .unwrap();

                let balance_rpc = account_balance.get("balance").unwrap();

                let mut balance = Amount::raw(0);

                let mut link = [0; 32];

                if action == "receive" {
                    let receivable = rpc_client
                        .receivable(
                            "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                            1,
                        )
                        .await
                        .unwrap();

                    println!("{:?}", receivable);

                    let blocks = receivable
                        .get("blocks")
                        .ok_or_else(|| anyhow::anyhow!("Missing blocks field"))?
                        .as_object()
                        .ok_or_else(|| anyhow::anyhow!("Blocks field is not an object"))?;

                    let (key, value) = blocks.iter().next().unwrap();

                    let link_decoded = hex::decode(key).unwrap();
                    link.copy_from_slice(&link_decoded);

                    balance = Amount::raw(str::parse(&value.as_str().unwrap()).unwrap())
                        + Amount::raw(str::parse(balance_rpc.as_str().unwrap()).unwrap())
                } else {
                    let link_decoded =
                        Account::decode_account(destination.as_ref().unwrap()).unwrap();
                    link.copy_from_slice(&link_decoded.0);

                    balance = Amount::raw(str::parse(balance_rpc.as_str().unwrap()).unwrap())
                        - Amount::raw(amount.unwrap())
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

                println!("hash: {:?}", tx_hash);

                let response = rpc_client.work_generate(previous_str).await.unwrap();

                let work_str = response.get("work").unwrap().as_str().unwrap();

                let work = u64_from_hex_str(work_str).unwrap();

                println!("work: {}", work);

                let block = StateBlock {
                    work,
                    signature,
                    hashables,
                    hash,
                    sideband: None,
                };

                /*{
                "type": "state",
                "account": "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                "previous": "0000000000000000000000000000000000000000000000000000000000000000",
                "representative": "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                "balance": "100000000000000000000000000000000",
                "link": "00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C",
                "signature": "3c80be057a168959f78479b406ffca34542a2dc1a526f70813d010767706ee3819f04fddc9cee4f36805f9177832e9dd7b9eb66837c53f1ce4ee53e0459f5409",
                "work": "de7c32d590b43708"
                }*/

                /*let json_data = r#"{
                  "type": "open",
                  "source": "00BE006B5F8E4133D63E5616727E1DDEC135ECD8AF71616A676369579E5A182C",
                  "representative": "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                  "account": "nano_39cfzwuamk4ca4d9kawsyogc7pricoj361je4hxthm8xyu76r8bizhag4c3q",
                  "work": "de7c32d590b43708",
                  "signature": "ef7c078b0855ac5f2e79a85ca13e3708ebc0778372bffdfd514ee4efb3fbcb34816fa111cbc08751b7d5cfa99fade74c3929772fddf8f310ab06c8095a773c04"
                }"#;

                let open_block: OpenBlock = serde_json::from_str(json_data).unwrap();*/

                let serialized = serde_json::to_string_pretty(&block).unwrap();

                //println!("block: {:?}", serialized);

                //println!("hash: {:?}", block.hash());

                let process = rpc_client
                    .process(action, &serde_json::to_string_pretty(&block).unwrap())
                    .await
                    .unwrap();

                println!("process: {:?}", process);

                /*let response = rpc_client
                .account_info_rpc(
                    "nano_1bnano1dnhc356frb1owg4mhi4r47j1i15yq8nuyyso8fg64ux9kdxzmae5g",
                )
                .await?;

                println!("{:?}", response);*/
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
        action: String,
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
        action: String,
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
