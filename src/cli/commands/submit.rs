use std::str::FromStr;
use crate::{cli::{commands::value_into_composite, errors::CliError}, files::FilePaths};
use schnorrkel::Signature;
use serde_json::from_str;
use subxt::{
    tx,
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::polkadot::PolkadotExtrinsicParamsBuilder,
    utils::{AccountId32, MultiSignature},
    OnlineClient, PolkadotConfig,
};
use tokio::fs::read_to_string;

pub async fn submit_threshold_extrinsic(
    files: String,
) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let threshold_public_key_string = read_to_string(file_paths.threshold_public_key()).await?;
	let account_id = AccountId32::from_str(from_str(&threshold_public_key_string)?)
        .map_err(|e| CliError::Custom(e.to_string()))?;

    let signature_string = read_to_string(file_paths.signature()).await?;
    let signature_bytes: Vec<u8> = from_str(&signature_string)?;
    let group_signature = Signature::from_bytes(&signature_bytes)?;

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
    
    let call_name = extrinsic_info
        .get("call_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'call_name' in extrinsic info".to_string()))?;

    let call_data = extrinsic_info
        .get("call_data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CliError::Custom("Missing 'call_data' in extrinsic info".to_string()))?;
    
    let value = scale_value::stringify::from_str(&call_data).0.unwrap();
    let value_as_composite = value_into_composite(value);
    
    let call = tx::dynamic(pallet, call_name, value_as_composite);

    let client = OnlineClient::<PolkadotConfig>::new().await?;
    let rpc_client = RpcClient::from_url(url).await?;
    let legacy_rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client);
    let nonce = legacy_rpc.system_account_next_index(&account_id).await?;

    let params = PolkadotExtrinsicParamsBuilder::new().nonce(nonce).build();
    let partial_extrinsic = client.tx().create_partial_signed_offline(&call, params)?;

    let signature = subxt_signer::sr25519::Signature(group_signature.to_bytes());

    let extrinsic = partial_extrinsic.sign_with_address_and_signature(
        &account_id.into(),
        &MultiSignature::Sr25519(signature.0),
    );

    extrinsic.submit().await?;

    println!("Submission of threshold extrinsic was completed successfully!");

    Ok(())
}