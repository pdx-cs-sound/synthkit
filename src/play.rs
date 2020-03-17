// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use portaudio as pa;

use std::error::Error;

use crate::*;

/// Number of samples for a blocking write.
/// XXX This has been carefully tuned to work
/// around a `portaudio` bug: I do not suggest
/// changing it.
const OUT_FRAMES: usize = 16;

/// Gather samples and post for playback.
pub fn play(mut samples: Stream) -> Result<(), Box<Error>> {

    // Create and initialize audio output.
    pa::initialize()?;
    let stream = pa::stream::Stream::open_default(
        0, // 0 input channels.
        1, // 1 output channel.
        SAMPLE_RATE as f64,
        pa::stream::FRAMES_PER_BUFFER_UNSPECIFIED, // Least possible buffer.
        None // No calback.
    )?;
    stream.start()?;

    let mut out = [0.0; OUT_FRAMES];
    let mut done = false;
    loop {
        for i in 0..OUT_FRAMES {
            match samples.next() {
                Some(s) => out[i] = s,
                None => {
                    for j in i..OUT_FRAMES {
                        out[j] = 0.0;
                    }
                    done = true;
                },
            }
        }
        stream.write(&out)?;
        if done {
            break;
        }
    }

    stream.stop()?;
    pa::terminate()?;
    Ok(())
}
