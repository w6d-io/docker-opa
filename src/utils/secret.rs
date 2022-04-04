use std::result;

use anyhow::anyhow;
use harsh::Harsh;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

use crate::config::CONFIG;

pub type Result<T, E = SecretError> = result::Result<T, E>;

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
///Enum containing secret error type.
pub enum SecretError {
    #[error("failed to decode the hash")]
    ErrorInvalidHash,
    #[error("the hash is diferent from the hash generated from the payload")]
    ErrorWrongHash(#[from] anyhow::Error),
    #[error("failed to build the hashid")]
    ErrorHarsh(#[from] harsh::BuildError),
}

///Create a hashid from the project id.
pub async fn get_secret(repository_id: u32) -> Result<String> {
    let config = CONFIG.read().await;
    let hash = Harsh::builder()
        .salt(&config.salt as &str)
        .length(config.salt_length)
        .build()?;
    let id = hash.encode(&[repository_id as u64]);
    Ok(id)
}

///Validate a github payload with its hash.
///will fail if:
/// - the hash is not a valid hex encoded String
/// - the canot be validated
pub fn validate_github_payload_sha256(hash: &str, body: &str, secret: &str) -> Result<()> {
    info!("Valitating github payload.");
    let cleaned_hash = &hash[7..hash.len()];
    let hash_bytes = match hex::decode(cleaned_hash) {
        Ok(result) => result,
        Err(e) => {
            warn!("Error decoding hash: {e}");
            return Err(SecretError::ErrorInvalidHash);
        }
    };
    let mut validation_hash = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|e| SecretError::ErrorWrongHash(anyhow!(e)))?;
    validation_hash.update(body.as_bytes());
    if let Err(e) = validation_hash.verify_slice(&hash_bytes) {
        error!("failed to validate payload: {e}");
        return Err(SecretError::ErrorWrongHash(anyhow!(e)));
    }
    Ok(())
}

///Validate a gitlab payload with its hash.
///fail if the hash canot be validated.
pub fn validate_gitlab_payload(hash: &str, secret: &str) -> Result<()> {
    info!("Valitating gitlab payload.");
    if hash != secret {
        let e = "the hashid form the header is diferent from the hashid generated";
        return Err(SecretError::ErrorWrongHash(anyhow!(e)));
    }
    Ok(())
}

#[cfg(test)]
///Module containing useful fonction for testig secret.
pub mod secret_test_utils {
    use anyhow::Result;
    use hex::encode;

    use super::*;

    ///Generate a hash from the given id and payload.
    pub fn generate_hash(id: u32, payload: &str) -> Result<String> {
        let secret = get_secret(id)?;
        let mut hash = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .map_err(|e| SecretError::ErrorWrongHash(anyhow!(e)))?;
        hash.update(payload.as_bytes());
        let bytes_hash = hash.finalize().into_bytes();
        Ok(encode(bytes_hash))
    }
}

#[cfg(test)]
mod test_secret {
    use super::*;

    #[test]
    fn test_validate_github_payload_sha256() {
        let id = 1;
        let payload = "bonjour";
        let secret = get_secret(id).unwrap();
        let hash = "sha256=".to_owned() + &secret_test_utils::generate_hash(id, payload).unwrap();
        let validate = validate_github_payload_sha256(&hash, payload, &secret);
        assert!(validate.is_ok())
    }

    #[test]
    fn test_validate_github_payload_sha256_bad_hash() {
        let id = 1;
        let payload = "bonjour";
        let secret = get_secret(id).unwrap();
        let hash = secret_test_utils::generate_hash(id, payload).unwrap();
        let validate = validate_github_payload_sha256(&hash, payload, &secret);
        assert!(validate.is_err())
    }

    #[test]
    fn test_validate_gitlab_payload() {
        let expected_hash = "pvGMmkr7NY5A61Q3wedgpzLRq2X8xJjEKyZlgPVO";
        let id = 1;
        let secret = get_secret(id).unwrap();
        let validate = validate_gitlab_payload(&secret, expected_hash);
        assert!(validate.is_ok())
    }

    #[test]
    fn test_validate_gitlab_payload_bad_hash() {
        let id = 1;
        let secret = get_secret(id).unwrap();
        let validate = validate_gitlab_payload("bonjour", &secret);
        assert!(validate.is_err())
    }

    #[test]
    fn test_get_secret() {
        let id = 1;
        let hash = get_secret(id).unwrap();
        info!("examples hash generated:{hash}");
        assert_eq!(hash, "pvGMmkr7NY5A61Q3wedgpzLRq2X8xJjEKyZlgPVO")
    }
}
