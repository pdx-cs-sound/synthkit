// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

pub struct Mixer {
    streams: Vec<Box<Iterator<Item=f32>>>,
    nstreams: usize,
}

impl Mixer {
    pub fn new(streams: Vec<Box<Iterator<Item=f32>>>) -> Self {
        let nstreams = streams.len();
        Self { streams, nstreams }
    }
}

/// Iterator over simultaneous streams of samples that adds
/// them to get a result.
impl Iterator for Mixer {
    type Item = f32;

    // Get the next mixed sample. We do not assume that
    // the streams are infinite.
    fn next(&mut self) -> Option<f32> {
        let mut streams = Vec::with_capacity(self.streams.len());
        let mut result = None;
        for st in &mut self.streams {
            let v = st.next();
            match v {
                None => (),
                Some(s) => {
                    match result {
                        None => result = Some(s),
                        Some(t) => result = Some(s + t),
                    }
                    streams.push(*st);
                },
            }
        }
        self.streams = streams;
        result.map(|s| s / self.nstreams as f32)
    }
}
