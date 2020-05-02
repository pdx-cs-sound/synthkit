// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Workaround for `Vec::retain()` passing `&T` instead of
// `&mut T`. See RFC #2160 and issue #25477 for discussion
// of inclusion of this in `std` (looks like it won't be),
// and issue #43244 tracking `Vec::drain_filter()`, which
// is in nightly as a more general proposed replacement,
// but currently has stabilization issues.
use std::collections::HashMap;

use crate::*;

/// A sample "mixer" that adds values from streams of
/// samples (currently always associated with a key) and
/// scales appropriately to get output samples.  Implemented
/// as an unbounded iterator: will return `Some(0.0)` when
/// no sample streams are available.
#[derive(Debug)]
pub struct Mixer<'a> {
    /// Held key indexes and samples.
    held: HashMap<usize, Samples<'a>>,
    /// Current mixer gain value.
    gain: f32,
}

impl<'a> Default for Mixer<'a> {
    fn default() -> Self {
        Mixer::new()
    }
}

/// Max voices before AGC kicks in.
const AGC_VOICES: usize = 8;
/// Mixer gain before AGC kicks in.
const LINEAR_GAIN: f32 = 0.1;

impl<'a> Mixer<'a> {
    /// New mixer with no streams.
    pub fn new() -> Self {
        Self {
            held: HashMap::with_capacity(128),
            gain: LINEAR_GAIN,
        }
    }

    /// New mixer with initial streams.
    pub fn with_streams(streams: Vec<(usize, Samples<'a>)>) -> Self {
        let mut result = Self::new();
        for (k, s) in streams.into_iter() {
            result.add_key(k, s);
        }
        result
    }

    /// Add a stream to the mixer.
    pub fn add_key(&mut self, key: usize, st: Samples<'a>) {
        let was_held = self.held.insert(key, st);
        assert!(was_held.is_none());
        self.agc();
    }

    /// Remove a stream from the mixer by key.
    pub fn remove_key(&mut self, key: usize) {
        self.held.remove(&key);
    }

    /// Remove all streams from the mixer.
    pub fn clear(&mut self) {
        self.held.clear();
    }

    /// Adjust the gain to avoid clipping while preserving
    /// some linearity. Essentially a compressor.
    fn agc(&mut self) {
        let nstreams = self.held.len();
        self.gain = if nstreams <= AGC_VOICES {
            LINEAR_GAIN
        } else {
            LINEAR_GAIN * AGC_VOICES as f32 / nstreams as f32
        };
    }
}

/// Iterator over simultaneous streams of samples that adds
/// them to get a result.
impl<'a> Iterator for Mixer<'a> {
    type Item = f32;

    // Get the next mixed sample. We do not assume that the
    // input streams are infinite, but the output stream is.
    fn next(&mut self) -> Option<f32> {
        let mut result = 0.0;
        let mut finished = Vec::new();
        for (k, st) in self.held.iter_mut() {
            let s = st.next();
            match s {
                Some(s) => result += s,
                None => finished.push(*k),
            }
        }
        for k in finished {
            self.remove_key(k);
        }
        self.agc();
        Some(result * self.gain)
    }
}
