mod cli;

use anyhow::bail;
use anyhow::Result;
use blake2::digest::Update;
use blake2::digest::VariableOutput;
use blake2::Blake2bVar;
use clap::{Parser, Subcommand};
use cli::Cli;
use ed25519_dalek::VerifyingKey;
use olaf::{
    frost::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
    simplpedpop::{AllMessage, SPPOutput, SPPOutputMessage},
    SigningKeypair,
};
use primitive_types::U512;
use rand_core::OsRng;
use reqwest::Url;
use serde::ser::SerializeStruct;
use serde_json::from_str;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    sync::{Arc, RwLock},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let mut rng = OsRng;
    let keypair1 = SigningKeypair::generate(&mut rng);
    let keypair2 = SigningKeypair::generate(&mut rng);

    println!("keypair1 sk: {:?}", keypair1.secret_key);
    println!("keypair1 pk: {:?}", keypair1.verifying_key.as_bytes());
    println!("keypair2 sk: {:?}", keypair2.secret_key);
    println!("keypair2 pk: {:?}", keypair2.verifying_key.as_bytes());*/

    let cli = Cli::parse();
    cli.run()
}

#[derive(Clone, Debug)]
pub struct StateBlock {
    pub work: u64,
    pub signature: Signature,
    pub hashables: StateHashables,
    pub hash: LazyBlockHash,
    pub sideband: Option<BlockSideband>,
}

impl StateBlock {
    fn hash(&self) -> BlockHash {
        self.hash.hash(&self.hashables)
    }
}

impl From<&StateHashables> for BlockHash {
    fn from(hashables: &StateHashables) -> Self {
        let mut preamble = [0u8; 32];
        preamble[31] = BlockType::State as u8;
        BlockHashBuilder::new()
            .update(preamble)
            .update(hashables.account.0)
            .update(hashables.previous.as_bytes())
            .update(hashables.representative.0)
            .update(hashables.balance.raw.to_be_bytes())
            .update(hashables.link.as_bytes())
            .build()
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BlockType {
    Invalid = 0,
    NotABlock = 1,
    LegacySend = 2,
    LegacyReceive = 3,
    LegacyOpen = 4,
    LegacyChange = 5,
    State = 6,
}

impl serde::Serialize for StateBlock {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Block", 9)?;
        state.serialize_field("type", "state")?;
        state.serialize_field("account", &self.hashables.account)?;
        state.serialize_field("previous", &self.hashables.previous)?;
        state.serialize_field("representative", &self.hashables.representative)?;
        state.serialize_field("balance", &self.hashables.balance.to_string_dec())?;
        state.serialize_field("link", &self.hashables.link.encode_hex())?;
        state.serialize_field("link_as_account", &Account(self.hashables.link.0))?;
        state.serialize_field("signature", &self.signature)?;
        state.serialize_field("work", &to_hex_string(self.work))?;
        state.end()
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct StateHashables {
    // Account# / public key that operates this account
    // Uses:
    // Bulk signature validation in advance of further ledger processing
    // Arranging uncomitted transactions by account
    pub account: Account,

    // Previous transaction in this chain
    pub previous: BlockHash,

    // Representative of this account
    pub representative: Account,

    // Current balance of this account
    // Allows lookup of account balance simply by looking at the head block
    pub balance: Amount,

    // Link field contains source block_hash if receiving, destination account if sending
    pub link: Link,
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

    fn hash(&self) -> BlockHash {
        self.hash.hash(&self.hashables)
    }
}

impl serde::Serialize for OpenBlock {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Block", 6)?;
        state.serialize_field("type", "open")?;
        state.serialize_field("source", &self.hashables.source)?;
        state.serialize_field("representative", &self.hashables.representative)?;
        state.serialize_field("account", &self.hashables.account)?;
        state.serialize_field("work", &to_hex_string(self.work))?;
        state.serialize_field("signature", &self.signature)?;
        state.end()
    }
}

struct Request<'a> {
    action: &'a str,
    json_block: &'a str,
    subtype: &'a str,
    block: &'a str,
}

impl<'a> serde::Serialize for Request<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Request", 4)?;
        state.serialize_field("action", self.action)?;
        state.serialize_field("json_block", self.json_block)?;
        state.serialize_field("subtype", self.subtype)?;
        state.serialize_field("block", self.block)?;
        state.end()
    }
}

impl<'a> Request<'a> {
    fn new(subtype: &'a str, block: &'a str) -> Self {
        Request {
            action: "process",
            json_block: "true",
            subtype,
            block,
        }
    }
}

pub fn to_hex_string(i: u64) -> String {
    format!("{:016X}", i)
}

pub fn u64_from_hex_str(s: impl AsRef<str>) -> Result<u64, ParseIntError> {
    u64::from_str_radix(s.as_ref(), 16)
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

impl std::ops::Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Amount::raw(self.raw + rhs.raw)
    }
}

impl Amount {
    pub fn to_string_dec(self) -> String {
        self.raw.to_string()
    }

    pub fn decode_hex(s: impl AsRef<str>) -> Result<Self> {
        let value = u128::from_str_radix(s.as_ref(), 16)?;
        Ok(Amount::raw(value))
    }

    pub const fn raw(value: u128) -> Self {
        Self { raw: value }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Signature {
    bytes: [u8; 64],
}

use std::fmt::Write as _;

impl Signature {
    pub fn encode_hex(&self) -> String {
        let mut result = String::with_capacity(128);
        for byte in self.bytes {
            write!(&mut result, "{:02X}", byte).unwrap();
        }
        result
    }
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.encode_hex())
    }
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
pub struct Link([u8; 32]);

impl Link {
    pub fn encode_hex(&self) -> String {
        use std::fmt::Write;
        let mut result = String::with_capacity(64);
        for &byte in self.as_bytes() {
            write!(&mut result, "{:02X}", byte).unwrap();
        }
        result
    }

    pub fn as_bytes(&'_ self) -> &'_ [u8; 32] {
        &self.0
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord, Debug)]
pub struct Account([u8; 32]);

impl Account {
    pub fn encode_account(&self) -> String {
        let mut number = U512::from_big_endian(&self.0);
        let check = U512::from_little_endian(&self.account_checksum());
        number <<= 40;
        number |= check;

        let mut result = String::with_capacity(65);

        for _i in 0..60 {
            let r = number.byte(0) & 0x1f_u8;
            number >>= 5;
            result.push(account_encode(r));
        }
        result.push_str("_onan"); // nano_
        result.chars().rev().collect()
    }

    fn account_checksum(&self) -> [u8; 5] {
        let mut check = [0u8; 5];
        let mut blake = Blake2bVar::new(check.len()).unwrap();
        blake.update(&self.0);
        blake.finalize_variable(&mut check).unwrap();

        check
    }

    pub fn decode_account(source: impl AsRef<str>) -> Result<Account> {
        EncodedAccountStr(source.as_ref()).to_u512()?.to_account()
    }
}

struct EncodedAccountStr<'a>(&'a str);

impl<'a> EncodedAccountStr<'a> {
    fn to_u512(&self) -> Result<EncodedAccountU512> {
        //if !self.is_valid() {
        //bail!("invalid account string");
        //}

        let mut number = U512::default();
        for character in self.chars_after_prefix() {
            match self.decode_byte(character) {
                Some(byte) => {
                    number <<= 5;
                    number = number + byte;
                }
                None => bail!("invalid hex string"),
            }
        }
        Ok(EncodedAccountU512(number))
    }

    fn decode_byte(&self, character: char) -> Option<u8> {
        if character.is_ascii() {
            let character = character as u8;
            if (0x30..0x80).contains(&character) {
                let byte: u8 = account_decode(character);
                if byte != b'~' {
                    return Some(byte);
                }
            }
        }

        None
    }

    fn chars_after_prefix(&'_ self) -> impl Iterator<Item = char> + '_ {
        self.0.chars().skip(5)
    }
}

fn account_decode(value: u8) -> u8 {
    let mut result = ACCOUNT_REVERSE[(value - 0x30) as usize] as u8;
    if result != b'~' {
        result -= 0x30;
    }
    result
}

const ACCOUNT_REVERSE: &[char] = &[
    '~', '0', '~', '1', '2', '3', '4', '5', '6', '7', '~', '~', '~', '~', '~', '~', '~', '~', '~',
    '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~',
    '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '~', '8', '9', ':', ';', '<', '=', '>', '?',
    '@', 'A', 'B', '~', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', '~', 'L', 'M', 'N', 'O', '~',
    '~', '~', '~', '~',
];

struct EncodedAccountU512(U512);

impl EncodedAccountU512 {
    fn account_bytes(&self) -> [u8; 32] {
        let mut bytes_512 = [0u8; 64];
        (self.0 >> 40).to_big_endian(&mut bytes_512);
        let mut bytes_256 = [0u8; 32];
        bytes_256.copy_from_slice(&bytes_512[32..]);
        bytes_256
    }

    fn to_account(&self) -> Result<Account> {
        let account = Account(self.account_bytes());
        /*if account.account_checksum() == self.checksum_bytes() {
            Ok(account)
        } else {
            Err(anyhow!("invalid checksum"))
            }*/
        Ok(account)
    }
}

const ACCOUNT_LOOKUP: &[char] = &[
    '1', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'w', 'x', 'y', 'z',
];

fn account_encode(value: u8) -> char {
    ACCOUNT_LOOKUP[value as usize]
}

impl serde::Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.encode_account())
    }
}

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

    pub fn as_bytes(&'_ self) -> &'_ [u8; 32] {
        &self.0
    }

    pub fn encode_hex(&self) -> String {
        use std::fmt::Write;
        let mut result = String::with_capacity(64);
        for &byte in self.as_bytes() {
            write!(&mut result, "{:02X}", byte).unwrap();
        }
        result
    }
}

impl serde::Serialize for BlockHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.encode_hex())
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

    /* {
      "action": "process",
      "json_block": "true",
      "subtype": "send",
      "block": {
        "type": "state",
        "account": "nano_1qato4k7z3spc8gq1zyd8xeqfbzsoxwo36a45ozbrxcatut7up8ohyardu1z",
        "previous": "6CDDA48608C7843A0AC1122BDD46D9E20E21190986B19EAC23E7F33F2E6A6766",
        "representative": "nano_3pczxuorp48td8645bs3m6c3xotxd3idskrenmi65rbrga5zmkemzhwkaznh",
        "balance": "40200000001000000000000000000000000",
        "link": "87434F8041869A01C8F6F263B87972D7BA443A72E0A97D7A3FD0CCC2358FD6F9",
        "link_as_account": "nano_33t5by1653nt196hfwm5q3wq7oxtaix97r7bhox5zn8eratrzoqsny49ftsd",
        "signature": "A5DB164F6B81648F914E49CAB533900C389FAAD64FBB24F6902F9261312B29F730D07E9BCCD21D918301419B4E05B181637CF8419ED4DCBF8EF2539EB2467F07",
        "work": "000bc55b014e807d"
      }
    } */

    pub async fn receivable(&self, account: &str, count: u32) -> Result<serde_json::Value> {
        let request = json!({
            "action": "receivable",
            "account": account,
            "count": count,
            "threshold": 1,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn account_balance(&self, account: &str) -> Result<serde_json::Value> {
        let request = json!({
            "action": "account_balance",
            "account": account,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn account_history(&self, account: &str, count: u32) -> Result<serde_json::Value> {
        let request = json!({
            "action": "account_history",
            "account": account,
            "count": count,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn work_generate(&self, hash: &str) -> Result<serde_json::Value> {
        let request = json!({
            "action": "work_generate",
            "hash": hash,
        });
        println!("request: {:?}", request);
        self.rpc_request(&request).await
    }

    pub async fn process(&self, subtype: &str, block: &str) -> Result<serde_json::Value> {
        //let request = Request::new(subtype, block);
        //let request_json = serde_json::to_value(&request)?;

        let request_json = json!({
            "action": "process",
            "json_block": "false",
            "subtype": subtype,
            "block": block
        });

        println!("request: {:?}", request_json);
        self.rpc_request(&request_json).await
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

    pub async fn account_info_rpc(&self, account: &str) -> Result<AccountInfo> {
        let request = json!({
            "action": "account_info",
            "account": account
        });

        let json = self.rpc_request(&request).await?;

        Ok(AccountInfo {
            frontier: json["frontier"].as_str().unwrap().to_owned(),
            block_count: json["block_count"].as_str().unwrap().to_owned(),
            balance: json["balance"].as_str().unwrap().to_owned(),
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct AccountInfo {
    pub frontier: String,
    pub block_count: String,
    pub balance: String,
}
