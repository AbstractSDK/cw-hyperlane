pub mod bind;
pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

// version info for migration info
pub const CONTRACT_NAME: &str = "hpl-self-xyz-verifier";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
