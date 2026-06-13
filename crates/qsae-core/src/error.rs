use thiserror::Error;

pub type Result<T> = std::result::Result<T, QsaeError>;

#[derive(Error, Debug)]
pub enum QsaeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format: {0}")]
    Format(String),

    #[error("Invalid magic number: expected QSAE, got {0:?}")]
    InvalidMagic([u8; 4]),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("Checksum mismatch: computed {computed:#x}, expected {expected:#x}")]
    ChecksumMismatch { computed: u64, expected: u64 },

    #[error("Codec {0} not available")]
    CodecUnavailable(u8),

    #[error("Decompression error: {0}")]
    Decompression(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid quorum parameters: {0}")]
    InvalidQuorumParams(String),
}
