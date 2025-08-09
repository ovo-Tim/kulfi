use std::error::Error;
use std::fmt;

/// Error when parsing an ID52 string
#[derive(Debug, Clone)]
pub struct ParseId52Error {
    pub input: String,
    pub reason: String,
}

impl fmt::Display for ParseId52Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ID52 '{}': {}", self.input, self.reason)
    }
}

impl Error for ParseId52Error {}

/// Error when parsing a secret key from string
#[derive(Debug, Clone)]
pub struct ParseSecretKeyError {
    pub reason: String,
}

impl fmt::Display for ParseSecretKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid secret key: {}", self.reason)
    }
}

impl Error for ParseSecretKeyError {}

/// Error when creating keys from invalid byte arrays
#[derive(Debug, Clone)]
pub struct InvalidKeyBytesError {
    pub expected: usize,
    pub got: usize,
}

impl fmt::Display for InvalidKeyBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid key length: expected {} bytes, got {}",
            self.expected, self.got
        )
    }
}

impl Error for InvalidKeyBytesError {}

/// Error when signature verification fails
#[derive(Debug, Clone)]
pub struct SignatureVerificationError;

impl fmt::Display for SignatureVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signature verification failed")
    }
}

impl Error for SignatureVerificationError {}

/// Error when creating signature from invalid bytes
#[derive(Debug, Clone)]
pub struct InvalidSignatureBytesError {
    pub expected: usize,
    pub got: usize,
}

impl fmt::Display for InvalidSignatureBytesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid signature length: expected {} bytes, got {}",
            self.expected, self.got
        )
    }
}

impl Error for InvalidSignatureBytesError {}