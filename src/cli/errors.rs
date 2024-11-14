use core::fmt;
use hex::FromHexError;
use schnorrkel::{
    olaf::{multisig::errors::MultiSigError, simplpedpop::errors::SPPError},
    SignatureError,
};
use sp_core::crypto::PublicError;
use subxt::Error as SubxtError;

#[derive(Debug)]
pub enum CliError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    MultiSig(MultiSigError),
    Signature(SignatureError),
    Subxt(SubxtError),
    SimplPedPop(SPPError),
    Hex(FromHexError),
    Account(PublicError)
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliError::Io(err) => write!(f, "IO Error: {}", err),
            CliError::Serde(err) => write!(f, "Serde Error: {}", err),
            CliError::MultiSig(err) => write!(f, "MultiSig Error: {:?}", err),
            CliError::Signature(err) => write!(f, "Signature Error: {}", err),
            CliError::Subxt(err) => write!(f, "Subxt Error: {}", err),
            CliError::SimplPedPop(err) => write!(f, "SimplPedPop Error: {:?}", err),
            CliError::Hex(err) => write!(f, "Hex Error: {:?}", err),
            CliError::Account(err) => write!(f, "Account Error: {:?}", err),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::Io(error)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::Serde(error)
    }
}

impl From<MultiSigError> for CliError {
    fn from(error: MultiSigError) -> Self {
        CliError::MultiSig(error)
    }
}

impl From<SignatureError> for CliError {
    fn from(error: SignatureError) -> Self {
        CliError::Signature(error)
    }
}

impl From<SubxtError> for CliError {
    fn from(error: SubxtError) -> Self {
        CliError::Subxt(error)
    }
}

impl From<SPPError> for CliError {
    fn from(error: SPPError) -> Self {
        CliError::SimplPedPop(error)
    }
}

impl From<FromHexError> for CliError {
    fn from(error: FromHexError) -> Self {
        CliError::Hex(error)
    }
}

impl From<PublicError> for CliError {
    fn from(error: PublicError) -> Self {
        CliError::Account(error)
    }
}
