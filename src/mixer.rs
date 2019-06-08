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

type Streams<'a> = Vec<Box<Iterator<Item=f32> + 'a>>;

pub struct Mixer<'a> {
    streams: Streams<'a>,
    nstreams: usize,
}

impl<'a> Mixer<'a> {
    pub fn new(streams: Streams<'a>) -> Self {
        let nstreams = streams.len();
        Self { streams, nstreams }
    }
}


/// Iterator over simultaneous streams of samples that adds
/// them to get a result.
impl<'a> Iterator for Mixer<'a> {
    type Item = f32;

    // Get the next mixed sample. We do not assume that
    // the streams are infinite.
    fn next(&mut self) -> Option<f32> {
        let mut result = None;
        self.streams.retain_mut(|st| {
            match st.next() {
                Some(t) => {
                    result = result.map(|s| s + t).or_else(|| Some(t));
                    true
                },
                None => false,
            }
        });
        result.map(|s| s / (2.0 * self.nstreams as f32))
    }
}
