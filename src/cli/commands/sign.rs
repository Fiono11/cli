use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{
    olaf::{
        multisig::{SigningCommitments, SigningNonces},
        simplpedpop::SPPOutputMessage,
        SigningKeypair,
    },
    PublicKey,
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

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

pub async fn threshold_sign_round1(files: String) -> Result<(), CliError> {
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

pub async fn threshold_sign_round2(
    files: String,
    url: String,
    pallet: String,
    function: String,
    call_args: String,
    context: String,
) -> Result<(), CliError> {
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

    let client: OnlineClient<PolkadotConfig> = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url(url).await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);

    // Parse call arguments from JSON string
    let args: Vec<Value> = serde_json::from_str(&call_args)?;

    // Create the call with the given pallet, function, and arguments
    let call = subxt::dynamic::tx(&pallet, &function, args);

    let account_id = AccountId32(threshold_public_key.to_bytes());
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_tx = client.tx().create_partial_signed_offline(&call, params)?;
    let payload = partial_tx.signer_payload().to_vec();

    let signing_package = signing_share.sign(
        context.as_bytes().to_vec(),
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

    // Save extrinsic arguments to a file
    let extrinsic_args = serde_json::json!({
        "pallet": pallet,
        "function": function,
        "call_args": call_args,
    });
    let extrinsic_args_string = serde_json::to_string_pretty(&extrinsic_args)?;
    let mut extrinsic_args_file = File::create(file_paths.extrinsic_info()).await?;
    extrinsic_args_file
        .write_all(extrinsic_args_string.as_bytes())
        .await?;

    Ok(())
}

