mod backend;
mod local;
mod errors;

#[cfg(feature = "s3")]
mod s3;

pub use backend::StorageBackend;
pub use local::LocalBackend;
pub use errors::*;

#[cfg(feature = "s3")]
pub use s3::S3Backend;
