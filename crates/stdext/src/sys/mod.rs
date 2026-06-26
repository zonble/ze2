// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Platform abstractions.

#[cfg(unix)]
mod unix;
#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use std::fs::canonicalize;

#[cfg(unix)]
pub use unix::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(windows)]
pub use windows::*;
