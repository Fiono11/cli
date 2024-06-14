use anyhow::bail;
use anyhow::Result;
use blake2::digest::Update;
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use clap::{Parser, Subcommand};
use ed25519_dalek::VerifyingKey;
use olaf::{
    frost::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
    simplpedpop::{AllMessage, SPPOutput},
    SigningKeypair,
};
use rand_core::OsRng;
use reqwest::Url;
use serde_json::from_str;
use serde_json::json;
use std::str::FromStr;
use std::time::Duration;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, RwLock},
};

fn async main() -> Result<(), Box<dyn std::error::Error>> {
    /*let mut rng = OsRng;
    let keypair1 = SigningKeypair::generate(&mut rng);
    let keypair2 = SigningKeypair::generate(&mut rng);

    println!("keypair1 sk: {:?}", keypair1.secret_key);
    println!("keypair1 pk: {:?}", keypair1.verifying_key.as_bytes());
    println!("keypair2 sk: {:?}", keypair2.secret_key);
    println!("keypair2 pk: {:?}", keypair2.verifying_key.as_bytes());*/

    let cli = Cli::parse();

    match cli.command {
        Commands::SimplpedpopRound1 { files } => {
            let file_path: std::path::PathBuf = Path::new(&files).into();

            let secret_key_string =
                fs::read_to_string(file_path.join("contributor_secret_key.json")).unwrap();

            let secret_key_bytes: Vec<u8> = from_str(&secret_key_string).unwrap();

            let mut secret_key = [0; 32];
            secret_key.copy_from_slice(&secret_key_bytes);

            let mut keypair = SigningKeypair::from_secret_key(&secret_key);

            let recipients_string = fs::read_to_string(file_path.join("recipients.json")).unwrap();

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

            let mut all_message_file = File::create(file_path.join("all_messages.json")).unwrap();

            all_message_file
                .write_all(&all_message_json.as_bytes())
                .unwrap();
        }
        Commands::SimplpedpopRound2 { files } => {
            let file_path: std::path::PathBuf = Path::new(&files).into();

            let secret_key_string =
                fs::read_to_string(file_path.join("contributor_secret_key.json")).unwrap();

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
        Commands::FrostRound2 { files } => {
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
            let spp_output = SPPOutput::from_bytes(&output_bytes).unwrap();

            let source = BlockHash([0; 32]);
            let representative = Account([0; 32]);
            let account = Account([0; 32]);

            let hashables = OpenHashables {
                source,
                representative,
                account,
            };

            let hash = LazyBlockHash::new();
            let tx_hash = hash.hash(&hashables).0;

            let signing_package = signing_share
                .sign(&tx_hash, &spp_output, &signing_commitments, &signing_nonces)
                .unwrap();

            let signing_packages_vec = vec![signing_package.to_bytes()];

            let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec).unwrap();

            let mut signing_package_file =
                File::create(file_path.join("signing_packages.json")).unwrap();

            signing_package_file
                .write_all(signing_package_json.as_bytes())
                .unwrap();
        }
        Commands::FrostAggregate { files } => {
            let file_path: std::path::PathBuf = Path::new(&files).into();

            let signing_packages_string =
                fs::read_to_string(file_path.join("signing_packages.json")).unwrap();

            let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string).unwrap();

            let signing_packages: Vec<SigningPackage> = signing_packages_bytes
                .iter()
                .map(|signing_commitments| SigningPackage::from_bytes(signing_commitments).unwrap())
                .collect();

            let signature = crate::Signature {
                bytes: aggregate(&signing_packages).unwrap().to_bytes(),
            };

            let work = 0;

            let source = BlockHash([0; 32]);
            let representative = Account([0; 32]);
            let account = Account([0; 32]);

            let hashables = OpenHashables {
                source,
                representative,
                account,
            };

            let hash = LazyBlockHash::new();

            let rpc_client = RpcClient::new(Url::from_str("url").unwrap());

            let block = OpenBlock {
                work,
                signature,
                hashables,
                hash,
                sideband: None,
            };

            rpc_client.receive_block("wallet", account, block).await;
        }
    }
    Ok(())
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
        files: String,
    },
    FrostAggregate {
        #[arg(long)]
        files: String,
    },
}

#[derive(Clone, Debug)]
pub struct OpenBlock {
    pub work: u64,
    pub signature: Signature,
    pub hashables: OpenHashables,
    pub hash: LazyBlockHash,
    pub sideband: Option<BlockSideband>,
}

impl OpenBlock {
    pub fn new(
        source: BlockHash,
        representative: Account,
        account: Account,
        signing_share: &RawKey,
        pub_key: &Account,
        work: u64,
    ) -> Self {
        let hashables = OpenHashables {
            source,
            representative,
            account,
        };

        let hash = LazyBlockHash::new();
        let tx_hash = hash.hash(&hashables).0;
        let signature = Signature { bytes: [0; 64] };
        //let signature = sign_message(prv_key, pub_key, hash.hash(&hashables).0);

        Self {
            work,
            signature,
            hashables,
            hash,
            sideband: None,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct LazyBlockHash {
    // todo: Remove Arc<RwLock>? Maybe remove lazy hash calculation?
    hash: Arc<RwLock<BlockHash>>,
}

impl LazyBlockHash {
    pub fn new() -> Self {
        Self {
            hash: Arc::new(RwLock::new(BlockHash::zero())),
        }
    }
    pub fn hash(&'_ self, factory: impl Into<BlockHash>) -> BlockHash {
        let mut value = self.hash.read().unwrap();
        if value.is_zero() {
            drop(value);
            let mut x = self.hash.write().unwrap();
            let block_hash: BlockHash = factory.into();
            *x = block_hash;
            drop(x);
            value = self.hash.read().unwrap();
        }

        *value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockSideband {
    pub height: u64,
    pub timestamp: u64,
    /// Successor to the current block
    pub successor: BlockHash,
    pub account: Account,
    pub balance: Amount,
    pub details: BlockDetails,
    pub source_epoch: Epoch,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub enum Epoch {
    Invalid = 0,
    #[default]
    Unspecified = 1,
    Epoch0 = 2,
    Epoch1 = 3,
    Epoch2 = 4,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockDetails {
    pub epoch: Epoch,
    pub is_send: bool,
    pub is_receive: bool,
    pub is_epoch: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct Amount {
    raw: u128, // native endian!
}

#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Signature {
    bytes: [u8; 64],
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpenHashables {
    /// Block with first send transaction to this account
    pub source: BlockHash,
    pub representative: Account,
    pub account: Account,
}

impl From<&OpenHashables> for BlockHash {
    fn from(hashables: &OpenHashables) -> Self {
        BlockHashBuilder::new()
            .update(hashables.source.0)
            .update(hashables.representative.0)
            .update(hashables.account.0)
            .build()
    }
}

impl Default for BlockHashBuilder {
    fn default() -> Self {
        Self {
            blake: Blake2bVar::new(32).unwrap(),
        }
    }
}

pub struct BlockHashBuilder {
    blake: Blake2bVar,
}

impl BlockHashBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(mut self, data: impl AsRef<[u8]>) -> Self {
        self.blake.update(data.as_ref());
        self
    }

    pub fn build(self) -> BlockHash {
        let mut hash_bytes = [0u8; 32];
        self.blake.finalize_variable(&mut hash_bytes).unwrap();
        BlockHash(hash_bytes)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord, Debug)]
pub struct Account([u8; 32]);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord, Debug)]
pub struct BlockHash([u8; 32]);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord, Debug)]
pub struct RawKey([u8; 32]);

impl BlockHash {
    pub const fn zero() -> Self {
        Self([0; 32])
    }

    pub fn is_zero(&self) -> bool {
        self.0 == [0; 32]
    }
}

pub struct RpcClient {
    url: Url,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            client: reqwest::ClientBuilder::new()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    async fn rpc_request(&self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let result = self
            .client
            .post(self.url.clone())
            .json(request)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if let Some(error) = result.get("error") {
            bail!("node returned error: {}", error);
        }

        Ok(result)
    }

    pub async fn receive_block(&self, wallet: &str, destination: &str, block: &str) -> Result<()> {
        let request = json!({
            "action": "receive",
            "wallet": wallet,
            "account": destination,
            "block": block
        });
        self.rpc_request(&request).await?;
        Ok(())
    }
}
