use schnorrkel::{olaf::multisig::{aggregate, SigningPackage}, Signature};
use serde_json::from_str;
use tokio::{fs::{read_to_string, File}, io::AsyncWriteExt};
use crate::{cli::errors::CliError, files::FilePaths};

/// Aggregates at least t partial signatures into one threshold signature
pub async fn aggregate_threshold_signature(files: String) -> Result<(), CliError> {
    let file_paths = FilePaths::new(files);

    let signing_packages_string = read_to_string(file_paths.signing_packages()).await?;
    let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string)?;
    let signing_packages: Vec<SigningPackage> = signing_packages_bytes
        .iter()
        .map(|sp| SigningPackage::from_bytes(sp))
        .collect::<Result<_, _>>()?;

    let group_signature: Signature = aggregate(&signing_packages)?;
    let signature_json = serde_json::to_string(&group_signature.to_bytes().to_vec())?;
    let mut signature_file = File::create(file_paths.signature()).await?;
    signature_file.write_all(signature_json.as_bytes()).await?;

    println!("Aggregation of threshold signature was completed successfully!");
    println!("The threshold signature was written to: {:?}", file_paths.signature());

    Ok(())
}
