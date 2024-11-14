use crate::{cli::errors::CliError, files::FilePaths};
use schnorrkel::{
    PublicKey, Signature,
};
use serde_json::from_str;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::polkadot::PolkadotExtrinsicParamsBuilder,
    dynamic::Value,
    utils::{AccountId32, MultiAddress},
    OnlineClient, PolkadotConfig,
};
use tokio::fs::read_to_string;

pub async fn submit_threshold_extrinsic(
    files: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    // Read threshold_public_key
    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
    let threshold_public_key_bytes: Vec<u8> = from_str(&threshold_public_key_string)?;
    let threshold_public_key = PublicKey::from_bytes(&threshold_public_key_bytes)?;

    let account_id = AccountId32(threshold_public_key.to_bytes());
    println!("pk: {:?}", account_id.to_string());

    // Read group_signature
    let signature_string = read_to_string(file_paths.signature()).await?;
    let signature_bytes: Vec<u8> = from_str(&signature_string)?;
    let group_signature = Signature::from_bytes(&signature_bytes)?;

    // Read extrinsic arguments from file
    let extrinsic_info_string = read_to_string(file_paths.extrinsic_info()).await?;
    let extrinsic_info: serde_json::Value = serde_json::from_str(&extrinsic_info_string)?;

    let url = extrinsic_info
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'url' in extrinsic info".to_string()))?;
    let pallet = extrinsic_info
        .get("pallet")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'pallet' in extrinsic info".to_string()))?;
    let function = extrinsic_info
        .get("function")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'function' in extrinsic info".to_string()))?;
    let call_args_str = extrinsic_info
        .get("call_args")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'call_args' in extrinsic info".to_string()))?;
    let args: Vec<Value> = serde_json::from_str(call_args_str)?;

    let client = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url(url).await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;

    // Create the call with the given pallet, function, and arguments
    let call = subxt::dynamic::tx(pallet, function, args);
    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_extrinsic = client.tx().create_partial_signed_offline(&call, params)?;

    let signature = subxt_signer::sr25519::Signature(group_signature.to_bytes());
    let public_key: MultiAddress<AccountId32, _> =
        subxt_signer::sr25519::PublicKey(threshold_public_key.to_bytes()).into();

    let extrinsic = partial_extrinsic.sign_with_address_and_signature(&public_key, &signature.into());
    extrinsic.submit().await?;

    Ok(())
}