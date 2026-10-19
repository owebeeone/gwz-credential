//! Endpoint-local credential ownership and one explicit SSH file-key source.
//! Native SSH authentication remains an injected privileged endpoint port.
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

mod acquisition;
mod acquisition_driver;
mod acquisition_observation;
mod acquisition_state;
mod api;
mod driver;
mod explicit_key;
mod future;
mod key_container;
mod key_file;
mod key_worker;
mod observation;
mod output;
pub mod provider;
mod state;
mod values;

pub use acquisition::*;
pub use acquisition_observation::*;
pub use api::*;
pub use explicit_key::ExplicitKeyProvider;
pub use future::*;
pub use observation::*;
pub use output::*;
pub use provider::{CredentialPorts, ProviderSet};
pub use values::*;
