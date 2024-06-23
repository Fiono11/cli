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
