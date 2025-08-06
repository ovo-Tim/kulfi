use std::fmt;
use std::str::FromStr;

/// Newtype wrapper for public keys that handles encoding/decoding consistently
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PublicKey(InnerPublicKey);

/// Newtype wrapper for secret keys that handles encoding/decoding consistently
pub struct SecretKey(InnerSecretKey);

/// Newtype wrapper for signatures
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
    
    /// Create from id52 string (BASE32_DNSSEC encoding)
    pub fn from_id52(id52: &str) -> eyre::Result<Self> {
        let bytes = data_encoding::BASE32_DNSSEC
            .decode(id52.as_bytes())
            .map_err(|e| eyre::anyhow!("failed to decode id52: {:?}", e))?;
        if bytes.len() != 32 {
            return Err(eyre::anyhow!("id52 has invalid length: {}", bytes.len()));
        }
        let bytes: [u8; 32] = bytes.try_into().unwrap();
        Self::from_bytes(&bytes)
    }
    
    /// Export as raw bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        *self.0.as_bytes()
    }
    
    /// Export as id52 string (BASE32_DNSSEC encoding)
    pub fn to_id52(&self) -> String {
        data_encoding::BASE32_DNSSEC.encode(self.0.as_bytes())
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
            self.0.verify(message, &signature.0)
                .map_err(|e| eyre::anyhow!("signature verification failed: {}", e))
        }
        
        #[cfg(feature = "iroh")]
        {
            // iroh::PublicKey has a verify method 
            self.0.verify(message, &signature.0)
                .map_err(|e| eyre::anyhow!("signature verification failed: {}", e))
        }
    }
}

// Display implementation - uses id52 (BASE32_DNSSEC) encoding
impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_id52())
    }
}

// FromStr implementation - accepts id52 format
impl FromStr for PublicKey {
    type Err = eyre::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_id52(s)
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
    
    /// Get the id52 (base32 encoded public key)
    pub fn id52(&self) -> String {
        self.public_key().to_id52()
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