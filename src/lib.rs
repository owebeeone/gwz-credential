//! Portable endpoint-local credential ownership. Native providers are not included.
//!
//! Owners cannot be cloned into operation handles:
//! ```compile_fail
//! fn clone_owner(owner: &gwz_credential::CredentialOwner) {
//!     let _: gwz_credential::CredentialOwner = owner.clone();
//! }
//! ```
//! Secrets have no general byte getter:
//! ```compile_fail
//! let secret = gwz_credential::SecretBytes::new(vec![1, 2]).unwrap();
//! let bytes = secret.as_bytes();
//! ```
#![forbid(unsafe_code)]

mod api;
mod driver;
mod future;
mod observation;
mod output;
pub mod provider;
mod state;
mod values;

pub use api::*;
pub use future::*;
pub use observation::*;
pub use output::*;
pub use provider::{CredentialPorts, ProviderSet};
pub use values::*;
