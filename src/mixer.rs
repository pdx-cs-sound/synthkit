// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

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
        let old_streams = &mut self.streams;
        let mut new_streams = Vec::with_capacity(old_streams.len());
        let mut result = None;
        while let Some(mut st) = old_streams.pop() {
            let v = st.next();
            match v {
                None => (),
                Some(s) => {
                    match result {
                        None => result = Some(s),
                        Some(t) => result = Some(s + t),
                    }
                    new_streams.push(st);
                },
            }
        }
        *old_streams = new_streams;
        result.map(|s| s / self.nstreams as f32)
    }
}
