//! Local save storage and fresh-match entropy at the platform boundary.

use std::{error::Error, fmt};

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod native;

/// A single active save slot. The battle session validates the stored envelope.
#[expect(
    clippy::redundant_pub_crate,
    reason = "The private platform boundary supplies storage to the sibling battle manager."
)]
pub(crate) trait Store: Send + Sync {
    /// Read the slot, returning `None` when no save exists.
    fn read(&mut self) -> Result<Option<String>, StorageError>;
    /// Replace the slot without first deleting its previous contents.
    fn write(&mut self, data: &str) -> Result<(), StorageError>;
    /// Remove the slot; clearing an absent save succeeds.
    fn clear(&mut self) -> Result<(), StorageError>;
}

/// Platform failures contain diagnostics about the operation, never saved card data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The battle manager reports this private platform error to the player."
)]
pub(crate) enum StorageError {
    Unavailable(String),
    Read(String),
    Write(String),
    Clear(String),
    Quota,
    Entropy,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(reason) => write!(f, "Local saves are unavailable: {reason}"),
            Self::Read(reason) => write!(f, "Could not read the saved battle: {reason}"),
            Self::Write(reason) => write!(f, "Could not save the battle: {reason}"),
            Self::Clear(reason) => write!(f, "Could not clear the saved battle: {reason}"),
            Self::Quota => f.write_str("Local storage is full; free some space and retry saving."),
            Self::Entropy => f.write_str(
                "Could not generate a fresh match seed; check platform support and retry.",
            ),
        }
    }
}

impl Error for StorageError {}

/// Select browser-local or per-user native storage without reading or replacing a save.
#[expect(
    clippy::redundant_pub_crate,
    reason = "Application composition selects storage through the private platform boundary."
)]
pub(crate) fn default_store() -> Result<Box<dyn Store>, StorageError> {
    #[cfg(target_arch = "wasm32")]
    {
        browser::default_store()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::default_store()
    }
}

/// Obtain a fresh seed from platform entropy; never use wall-clock time as randomness.
#[expect(
    clippy::redundant_pub_crate,
    reason = "Fresh-match creation requests entropy at the private platform boundary."
)]
pub(crate) fn fresh_seed() -> Result<u64, StorageError> {
    #[cfg(target_arch = "wasm32")]
    {
        browser::fresh_seed()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        native::fresh_seed()
    }
}

/// Whether the browser document is visible and focused; native window focus is handled by Bevy.
#[cfg(target_arch = "wasm32")]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The private battle scheduler consumes platform activity."
)]
pub(crate) fn document_active() -> bool {
    browser::document_active()
}

#[cfg(not(target_arch = "wasm32"))]
#[expect(
    clippy::redundant_pub_crate,
    reason = "The private battle scheduler consumes platform activity."
)]
pub(crate) const fn document_active() -> bool {
    true
}
