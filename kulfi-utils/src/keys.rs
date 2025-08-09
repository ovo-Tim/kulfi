use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Newtype wrapper for public keys that handles encoding/decoding consistently
///
/// # Examples
///
/// ## Creating from ID52 string
///
/// ```
/// use kulfi_utils::PublicKey;
/// use std::str::FromStr;
///
/// let id52 = "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60";
/// let public_key = PublicKey::from_str(id52).unwrap();
///
/// // Convert back to ID52
/// assert_eq!(public_key.to_string(), id52);
/// ```
///
/// ## Verifying signatures
///
/// ```
/// use kulfi_utils::{SecretKey, PublicKey};
///
/// let secret_key = SecretKey::generate();
/// let public_key = secret_key.public_key();
///
/// let message = b"Hello, world!";
/// let signature = secret_key.sign(message);
///
/// // Verify with the public key
/// public_key.verify(message, &signature).expect("Valid signature");
/// ```
/// 
/// ## Serialization/Deserialization
/// 
/// ```
/// use kulfi_utils::PublicKey;
/// use std::str::FromStr;
/// 
/// let id52 = "i66fo538lfl5ombdf6tcdbrabp4hmp9asv7nrffuc2im13ct4q60";
/// let public_key = PublicKey::from_str(id52).unwrap();
/// 
/// // Serialize to JSON (uses ID52 format)
/// let json = serde_json::to_string(&public_key).unwrap();
/// assert_eq!(json, format!("\"{}\"", id52));
/// 
/// // Deserialize from JSON
/// let deserialized: PublicKey = serde_json::from_str(&json).unwrap();
/// assert_eq!(deserialized, public_key);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey(InnerPublicKey);

/// Newtype wrapper for secret keys that handles encoding/decoding consistently
///
/// # Examples
///
/// ## Generating and using a secret key
///
/// ```
/// use kulfi_utils::SecretKey;
///
/// // Generate a new random key
/// let secret_key = SecretKey::generate();
///
/// // Get the public key and ID52
/// let public_key = secret_key.public_key();
/// let id52 = secret_key.id52();
///
/// // Sign a message
/// let message = b"Hello, world!";
/// let signature = secret_key.sign(message);
/// ```
///
/// ## Parsing from string (hex or base32)
///
/// ```
/// use kulfi_utils::SecretKey;
/// use std::str::FromStr;
///
/// // Parse from hex string (64 characters)
/// let hex = "100d7e23f222267ba0be43855a262461b8a7718572edf58c56db912156d2bc25";
/// let secret_key = SecretKey::from_str(hex).unwrap();
///
/// // Display as hex
/// assert_eq!(format!("{}", secret_key), hex);
/// ```
///
/// ## Converting to/from bytes
///
/// ```
/// use kulfi_utils::SecretKey;
///
/// let secret_key = SecretKey::generate();
///
/// // Export as bytes
/// let bytes: [u8; 32] = secret_key.to_bytes();
///
/// // Import from bytes
/// let secret_key2 = SecretKey::from_bytes(&bytes);
/// assert_eq!(secret_key.id52(), secret_key2.id52());
/// ```
/// 
/// ## Serialization/Deserialization
/// 
/// ```
/// use kulfi_utils::SecretKey;
/// 
/// let secret_key = SecretKey::generate();
/// 
/// // Serialize to JSON (uses hex format)
/// let json = serde_json::to_string(&secret_key).unwrap();
/// 
/// // Deserialize from JSON
/// let deserialized: SecretKey = serde_json::from_str(&json).unwrap();
/// assert_eq!(deserialized.id52(), secret_key.id52());
/// ```
pub struct SecretKey(InnerSecretKey);

/// Newtype wrapper for signatures
///
/// # Examples
///
/// ## Creating and verifying a signature
///
/// ```
/// use kulfi_utils::{SecretKey, Signature};
///
/// // Generate a key pair
/// let secret_key = SecretKey::generate();
/// let public_key = secret_key.public_key();
///
/// // Sign a message
/// let message = b"Hello, world!";
/// let signature = secret_key.sign(message);
///
/// // Verify the signature
/// public_key.verify(message, &signature)
///     .expect("Signature should be valid");
/// ```
///
/// ## Converting signatures to/from bytes
///
/// ```
/// use kulfi_utils::{SecretKey, Signature};
///
/// let secret_key = SecretKey::generate();
/// let signature = secret_key.sign(b"test message");
///
/// // Convert to bytes for storage/transmission
/// let bytes: [u8; 64] = signature.to_bytes();
/// let vec: Vec<u8> = signature.to_vec();
///
/// // Or use From trait (consumes the signature)
/// let signature2 = secret_key.sign(b"test message");
/// let bytes2: [u8; 64] = signature2.into();
///
/// // Reconstruct from bytes
/// let signature3 = Signature::from_bytes(&bytes).unwrap();
/// ```
pub struct Signature(InnerSignature);

// Internal type aliases for the actual key types
#[cfg(not(feature = "iroh"))]
type InnerPublicKey = ed25519_dalek::VerifyingKey;

#[cfg(feature = "iroh")]
type InnerPublicKey = iroh::PublicKey;

#[cfg(not(feature = "iroh"))]
type InnerSecretKey = ed25519_dalek::SigningKey;

#[cfg(feature = "iroh")]
type InnerSecretKey = iroh::SecretKey;

// Both iroh and our code use ed25519_dalek::Signature for signatures.
// This is guaranteed by the current design: iroh::PublicKey and our code both expect ed25519_dalek::Signature,
// so conditional compilation is not needed here. If this ever changes in the future, revisit this alias.
type InnerSignature = ed25519_dalek::Signature;

// ============== PublicKey Implementation ==============

impl PublicKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> eyre::Result<Self> {
        #[cfg(not(feature = "iroh"))]
        {
            use ed25519_dalek::VerifyingKey;
            VerifyingKey::from_bytes(bytes)
                .map(PublicKey)
                .map_err(|e| eyre::anyhow!("invalid public key: {}", e))
        }

        #[cfg(feature = "iroh")]
        {
            Ok(PublicKey(iroh::PublicKey::from_bytes(bytes)?))
        }
    }

    /// Export as raw bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        *self.0.as_bytes()
    }

    /// Convert to inner type (consumes self)
    pub fn into_inner(self) -> InnerPublicKey {
        self.0
    }

    /// Create from the inner iroh type (only available with iroh feature)
    #[cfg(feature = "iroh")]
    pub fn from_iroh(key: iroh::PublicKey) -> Self {
        PublicKey(key)
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), eyre::Error> {
        #[cfg(not(feature = "iroh"))]
        {
            use ed25519_dalek::Verifier;
            self.0
                .verify(message, &signature.0)
                .map_err(|e| eyre::anyhow!("signature verification failed: {}", e))
        }

        #[cfg(feature = "iroh")]
        {
            // iroh::PublicKey has a verify method
            self.0
                .verify(message, &signature.0)
                .map_err(|e| eyre::anyhow!("signature verification failed: {}", e))
        }
    }
}

// Display implementation - uses id52 (BASE32_DNSSEC) encoding
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", data_encoding::BASE32_DNSSEC.encode(self.0.as_bytes()))
    }
}

// FromStr implementation - accepts id52 format (BASE32_DNSSEC)
impl FromStr for PublicKey {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_DNSSEC
            .decode(s.as_bytes())
            .map_err(|e| eyre::anyhow!("failed to decode id52: {:?}", e))?;
        if bytes.len() != 32 {
            return Err(eyre::anyhow!("id52 has invalid length: {}", bytes.len()));
        }
        let bytes: [u8; 32] = bytes.try_into().unwrap();
        Self::from_bytes(&bytes)
    }
}

// Serialize as ID52 string
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Deserialize from ID52 string
impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PublicKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// ============== SecretKey Implementation ==============

impl SecretKey {
    /// Generate a new random secret key
    pub fn generate() -> Self {
        #[cfg(not(feature = "iroh"))]
        {
            let mut rng = rand::rngs::OsRng;
            SecretKey(ed25519_dalek::SigningKey::generate(&mut rng))
        }

        #[cfg(feature = "iroh")]
        {
            let mut rng = rand::rngs::OsRng;
            SecretKey(iroh::SecretKey::generate(&mut rng))
        }
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        #[cfg(not(feature = "iroh"))]
        {
            SecretKey(ed25519_dalek::SigningKey::from_bytes(bytes))
        }

        #[cfg(feature = "iroh")]
        {
            SecretKey(iroh::SecretKey::from_bytes(bytes))
        }
    }

    /// Export as raw bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        #[cfg(not(feature = "iroh"))]
        {
            PublicKey(self.0.verifying_key())
        }

        #[cfg(feature = "iroh")]
        {
            PublicKey(self.0.public())
        }
    }

    /// Get the ID52 string of the public key
    /// 
    /// This is a convenience method equivalent to `self.public_key().to_string()`
    pub fn id52(&self) -> String {
        self.public_key().to_string()
    }

    /// Convert to inner type (consumes self)
    pub fn into_inner(self) -> InnerSecretKey {
        self.0
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        #[cfg(not(feature = "iroh"))]
        {
            use ed25519_dalek::Signer;
            Signature(self.0.sign(message))
        }

        #[cfg(feature = "iroh")]
        {
            Signature(self.0.sign(message))
        }
    }
}

// Display implementation - always uses hex encoding (matching iroh)
impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", data_encoding::HEXLOWER.encode(&self.to_bytes()))
    }
}

// FromStr implementation - accepts both hex and base32
impl FromStr for SecretKey {
    type Err = eyre::Error;

    /// Parse a secret key from a string.
    ///
    /// Accepts either:
    /// - Hex encoding (64 lowercase hex characters, e.g. as produced by `Display`)
    /// - Base32 encoding (52 uppercase base32 characters, no padding; for backward compatibility)
    ///
    /// Returns an error if the input is not valid hex or base32 encoding of a 32-byte secret key.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = if s.len() == 64 {
            // Hex encoding (our Display format and iroh's Display format)
            let mut result = [0u8; 32];
            data_encoding::HEXLOWER
                .decode_mut(s.as_bytes(), &mut result)
                .map_err(|e| eyre::anyhow!("failed to decode hex secret key: {:?}", e))?;
            result
        } else {
            // For backward compatibility, also try BASE32_NOPAD (iroh's alternative format)
            let input = s.to_ascii_uppercase();
            let mut result = [0u8; 32];
            data_encoding::BASE32_NOPAD
                .decode_mut(input.as_bytes(), &mut result)
                .map_err(|e| eyre::anyhow!("failed to decode base32 secret key: {:?}", e))?;
            result
        };

        Ok(SecretKey::from_bytes(&bytes))
    }
}

// Serialize as hex string
impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

// Deserialize from hex or base32 string
impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SecretKey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

// ============== Signature Implementation ==============

impl Signature {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8; 64]) -> eyre::Result<Self> {
        Ok(Signature(InnerSignature::from_bytes(bytes)))
    }

    /// Export as raw bytes
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }

    /// Convert to Vec<u8>
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

// Implement From for Vec<u8> conversion
impl From<Signature> for Vec<u8> {
    fn from(sig: Signature) -> Vec<u8> {
        sig.to_bytes().to_vec()
    }
}

// Implement From for [u8; 64] conversion
impl From<Signature> for [u8; 64] {
    fn from(sig: Signature) -> [u8; 64] {
        sig.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_serialization() {
        // Test PublicKey serialization/deserialization
        let secret_key = SecretKey::generate();
        let public_key = secret_key.public_key();
        let id52 = public_key.to_string();
        
        // Serialize to JSON
        let json = serde_json::to_string(&public_key).unwrap();
        assert_eq!(json, format!("\"{}\"", id52));
        
        // Deserialize from JSON
        let deserialized: PublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, public_key);
        
        // Test in a struct
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            key: PublicKey,
            name: String,
        }
        
        let test = TestStruct {
            key: public_key,
            name: "test".to_string(),
        };
        
        let json = serde_json::to_string(&test).unwrap();
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, test);
    }

    #[test]
    fn test_secret_key_serialization() {
        // Test SecretKey serialization/deserialization
        let secret_key = SecretKey::generate();
        let hex = format!("{}", secret_key);
        
        // Serialize to JSON
        let json = serde_json::to_string(&secret_key).unwrap();
        assert_eq!(json, format!("\"{}\"", hex));
        
        // Deserialize from JSON
        let deserialized: SecretKey = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id52(), secret_key.id52());
        assert_eq!(deserialized.to_bytes(), secret_key.to_bytes());
    }

    #[test]
    fn test_signature_bytes_conversion() {
        // Generate a key and sign a message
        let secret_key = SecretKey::generate();
        let message = b"test message";
        let signature = secret_key.sign(message);

        // Test to_bytes
        let bytes: [u8; 64] = signature.to_bytes();
        assert_eq!(bytes.len(), 64);

        // Test to_vec
        let vec: Vec<u8> = signature.to_vec();
        assert_eq!(vec.len(), 64);
        assert_eq!(&vec[..], &bytes[..]);

        // Test From trait for Vec<u8>
        let signature2 = secret_key.sign(message);
        let vec2: Vec<u8> = signature2.into();
        assert_eq!(vec2.len(), 64);

        // Test From trait for [u8; 64]
        let signature3 = secret_key.sign(message);
        let bytes3: [u8; 64] = signature3.into();
        assert_eq!(bytes3.len(), 64);

        // Test from_bytes roundtrip
        let signature4 = Signature::from_bytes(&bytes).unwrap();
        assert_eq!(signature4.to_bytes(), bytes);

        // Verify the signature works
        let public_key = secret_key.public_key();
        public_key
            .verify(message, &signature4)
            .expect("Signature should verify");
    }
}
