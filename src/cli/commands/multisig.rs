use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{
    olaf::{
        multisig::{aggregate, SigningCommitments, SigningNonces, SigningPackage},
        simplpedpop::SPPOutputMessage,
        SigningKeypair,
    },
    PublicKey, Signature,
};
use serde_json::from_str;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::polkadot::PolkadotExtrinsicParamsBuilder,
    dynamic::Value,
    utils::AccountId32,
    OnlineClient, PolkadotConfig,
};
use tokio::{
    fs::{read_to_string, File},
    io::AsyncWriteExt,
};

const CONTEXT: &[u8] = b"substrate";

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

// Multisig Round 1
pub async fn multisig_round1(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_share_string = read_to_string(file_paths.signing_share()).await?;
    let signing_share_bytes: Vec<u8> = from_str(&signing_share_string)?;
    let signing_share: SigningKeypair = SigningKeypair::from_bytes(&signing_share_bytes)?;

    let (signing_nonces, signing_commitments) = signing_share.commit();

    let signing_nonces_json = serde_json::to_string_pretty(&signing_nonces.to_bytes().to_vec())?;
    let mut signing_nonces_file = File::create(file_paths.signing_nonces()).await?;
    signing_nonces_file
        .write_all(signing_nonces_json.as_bytes())
        .await?;

    let signing_commitments_vec = vec![signing_commitments.to_bytes().to_vec()];
    let signing_commitments_json = serde_json::to_string_pretty(&signing_commitments_vec)?;

    let mut signing_commitments_file = File::create(file_paths.signing_commitments()).await?;
    signing_commitments_file
        .write_all(signing_commitments_json.as_bytes())
        .await?;

    Ok(())
}

// Multisig Round 2
pub async fn multisig_round2(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_commitments_string = read_to_string(file_paths.signing_commitments()).await?;
    let signing_commitments_bytes: Vec<Vec<u8>> = from_str(&signing_commitments_string)?;
    let signing_commitments: Vec<SigningCommitments> = signing_commitments_bytes
        .iter()
        .map(|sc| SigningCommitments::from_bytes(sc))
        .collect::<Result<_, _>>()?;

    let signing_nonces_string = read_to_string(file_paths.signing_nonces()).await?;
    let signing_nonces_bytes: Vec<u8> = from_str(&signing_nonces_string)?;
    let signing_nonces = SigningNonces::from_bytes(&signing_nonces_bytes)?;

    let signing_share_string = read_to_string(file_paths.signing_share()).await?;
    let signing_share_bytes: Vec<u8> = from_str(&signing_share_string)?;
    let signing_share = SigningKeypair::from_bytes(&signing_share_bytes)?;

    let output_string = read_to_string(file_paths.spp_output()).await?;
    let output_bytes: Vec<u8> = from_str(&output_string)?;
    let spp_output = SPPOutputMessage::from_bytes(&output_bytes)?;

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
    let threshold_public_key_bytes: Vec<u8> = from_str(&threshold_public_key_string)?;
    let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes)?;

    let client = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url("ws://127.0.0.1:9944").await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    let call = subxt::dynamic::tx("System", "remark", vec![Value::from_bytes("Hello there")]);

    let account_id = AccountId32(threshold_public_key.to_bytes());
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;
    let payload = partial_tx.signer_payload().to_vec();

    let signing_package = signing_share.sign(
        CONTEXT.to_vec(),
        payload,
        spp_output.spp_output(),
        signing_commitments,
        &signing_nonces,
    )?;

    let signing_packages_vec = vec![signing_package.to_bytes()];
    let signing_package_json = serde_json::to_string_pretty(&signing_packages_vec)?;

    let mut signing_package_file = File::create(file_paths.signing_packages()).await?;
    signing_package_file
        .write_all(signing_package_json.as_bytes())
        .await?;

    Ok(())
}

// Multisig Aggregate
pub async fn multisig_aggregate(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
    let threshold_public_key_bytes: Vec<u8> = from_str(&threshold_public_key_string)?;
    let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes)?;

    let account_id = AccountId32(threshold_public_key.to_bytes());
    println!("pk: {:?}", account_id.to_string());

    let signing_packages_string = read_to_string(file_paths.signing_packages()).await?;
    let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string)?;
    let signing_packages: Vec<SigningPackage> = signing_packages_bytes
        .iter()
        .map(|sp| SigningPackage::from_bytes(sp))
        .collect::<Result<_, _>>()?;

    let group_signature: Signature = aggregate(&signing_packages)?;
    let signature_json = serde_json::to_string_pretty(&group_signature.to_bytes().to_vec())?;
    let mut signature_file = File::create(file_paths.signature()).await?;
    signature_file.write_all(signature_json.as_bytes()).await?;

    Ok(())
}
