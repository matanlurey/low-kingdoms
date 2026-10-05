//! Shared desktop and browser presentation for Low Kingdoms.

pub mod app;

#[cfg(feature = "test-support")]
mod diagnostics;
mod platform;
mod screens;
mod ui;

#[cfg(test)]
mod tests;
