// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Music synthesizer components. This crate includes a
//! bunch of half-finished code and kludgy options, and is
//! not yet suitable for general use.

mod midi;
mod mixer;
mod play;
mod sampler;
mod wavio;

pub use midi::*;
pub use mixer::*;
pub use play::*;
pub use sampler::*;
pub use wavio::*;

/// The audio sample rate is currently fixed at 48000
/// samples per second. This constant will be made a
/// parameter somehow in some future crate version.
pub const SAMPLE_RATE: u32 = 48_000;
