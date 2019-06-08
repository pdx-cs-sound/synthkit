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
use retain_mut::RetainMut;

/// A stream of samples is just an iterator that returns
/// samples.
type Stream<'a> = Box<Iterator<Item=f32> + 'a>;

/// A sample "mixer" that adds values from streams
/// of samples and scales appropriately to get output samples.
/// Implemented as an unbounded iterator: will return `Some(0.0)`
/// when no sample streams are available.
pub struct Mixer<'a> {
    /// Active iterators for streams.
    streams: Vec<Stream<'a>>,
    /// Current mixer gain value.
    gain: f32,
}

/// Max voices before AGC kicks in.
const AGC_VOICES: usize = 8;
/// Mixer gain before AGC kicks in.
const LINEAR_GAIN: f32 = 0.1;

impl<'a> Mixer<'a> {
    /// New mixer with no streams.
    pub fn new() -> Self {
        Self { streams: vec![], gain: LINEAR_GAIN }
    }

    /// New mixer with initial streams.
    pub fn with_streams(streams: Vec<Stream<'a>>) -> Self {
        let mut mixer = Self::new();
        for st in streams {
            mixer.add(st);
        }
        mixer
    }

    /// Add a stream to the mixer.
    pub fn add(&mut self, st: Stream<'a>) {
        self.streams.push(st);
        self.agc();
    }

    /// Adjust the gain to avoid clipping while preserving
    /// some linearity. Essentially a compressor.
    fn agc(&mut self) {
        let nstreams = self.streams.len();
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
        let mut agc = false;
        self.streams.retain_mut(|st| {
            let s = st.next();
            match s {
                Some(s) => result += s,
                None => agc = true,
            }
            s.is_some()
        });
        if agc {
            self.agc();
        }
        Some(result * self.gain)
    }
}
