use schnorrkel::{olaf::multisig::{aggregate, SigningPackage}, PublicKey, Signature};
use serde_json::from_str;
use subxt::utils::AccountId32;
use tokio::{fs::{read_to_string, File}, io::AsyncWriteExt};
use crate::{cli::errors::CliError, files::FilePaths};

pub async fn aggregate_threshold_signature(files: String) -> Result<(), CliError> {
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
