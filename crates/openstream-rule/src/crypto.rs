use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Некорректный публичный ключ: {0}")]
    InvalidPublicKey(String),
    #[error("Некорректный формат подписи: {0}")]
    InvalidSignature(String),
    #[error("Ошибка верификации подписи: подпись не соответствует содержимому")]
    VerificationFailed,
}

/// Подписать байты манифеста приватным ключом автора
pub fn sign_bytes(data: &[u8], signing_key: &SigningKey) -> [u8; 64] {
    let signature: Signature = signing_key.sign(data);
    signature.to_bytes()
}

/// Проверить подлинность манифеста по публичному ключу Ed25519
pub fn verify_signature(
    data: &[u8],
    signature_bytes: &[u8; 64],
    public_key_bytes: &[u8; 32],
) -> Result<(), CryptoError> {
    let verifying_key = VerifyingKey::from_bytes(public_key_bytes)
        .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;

    let signature = Signature::from_bytes(signature_bytes);

    verifying_key
        .verify(data, &signature)
        .map_err(|_| CryptoError::VerificationFailed)?;

    Ok(())
}
