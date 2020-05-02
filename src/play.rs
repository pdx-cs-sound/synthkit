// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use std::error::Error;
use std::sync::Mutex;

use portaudio_rs as pa;

use crate::*;

/// Number of samples for a blocking write.
const OUT_FRAMES: usize = 240;

/// Gather samples and post for playback.
pub fn play<T>(samples: &Mutex<T>) -> Result<(), Box<dyn Error>>
where
    T: Iterator<Item = f32>,
{
    // Create and initialize audio output.
    pa::initialize()?;
    let stream = pa::stream::Stream::open_default(
        0, // 0 input channels.
        1, // 1 output channel.
        SAMPLE_RATE as f64,
        pa::stream::FRAMES_PER_BUFFER_UNSPECIFIED, // Least possible buffer.
        None,                                      // No calback.
    )?;
    stream.start()?;

    let mut out = [0.0; OUT_FRAMES];
    let mut done = false;
    loop {
        {
            // Be sure to unlock by dropping the stream guard
            // before blocking in `write()`.
            let mut samples = samples.lock().unwrap();
            for i in 0..OUT_FRAMES {
                match samples.next() {
                    Some(s) => out[i] = s,
                    None => {
                        for s in &mut out[i..OUT_FRAMES] {
                            *s = 0.0;
                        }
                        done = true;
                    }
                }
            }
        }
        stream.write(&out)?;
        if done {
            break;
        }
    }

    Ok(())
}
