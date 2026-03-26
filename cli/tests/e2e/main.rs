//! E2E test suite for aibox CLI.
//!
//! - Tier 1 (always run): appearance tests, config coverage tests
//! - Tier 2 (--features e2e): lifecycle, reset, migration, addon, doctor, smoke tests

pub mod mock_runtime;
pub mod runner;

// Tier 1 tests (fast, no container needed)
mod appearance;
mod config_coverage;
mod preview;

// Tier 2 tests (require e2e-runner companion container)
#[cfg(feature = "e2e")]
mod addon;
#[cfg(feature = "e2e")]
mod doctor;
#[cfg(feature = "e2e")]
mod lifecycle;
#[cfg(feature = "e2e")]
mod migration;
#[cfg(feature = "e2e")]
mod reset;
#[cfg(feature = "e2e")]
mod smoke;
#[cfg(feature = "e2e")]
mod update;
#[cfg(feature = "e2e")]
mod visual;
#[cfg(feature = "e2e")]
mod visual_keybindings;
