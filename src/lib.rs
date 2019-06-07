// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Music synthesizer components. This crate includes a
//! bunch of half-finished code and kludgy options, and is
//! not yet suitable for general use.

mod midi;
mod play;
mod wavio;
mod sampler;
mod mixer;
mod retain_mut;

pub use midi::*;
pub use play::*;
pub use wavio::*;
pub use sampler::*;
pub use mixer::*;
pub use retain_mut::*;

/// The audio sample rate is currently fixed at 48000
/// samples per second. This constant will be made a
/// parameter somehow in some future crate version.
pub const SAMPLE_RATE: u32 = 48_000;
