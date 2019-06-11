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
const OUT_FRAMES: usize = 12;

/// Gather samples and post for playback.
pub fn play(mut samples: Stream) -> Result<(), Box<Error>> {

    // Callback supplies portaudio with a requested chunk of samples.

    // Create and initialize audio output.
    let out = pa::PortAudio::new()?;
    let mut settings = out.default_output_stream_settings(
        1, // 1 channel.
        SAMPLE_RATE as f64,
        0_u32, // Least possible buffer.
    )?;
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut stream = out.open_blocking_stream(settings)?;

    stream.start()?;

    let mut playing = true;
    while playing {
        stream.write(OUT_FRAMES as u32, |out| {
            let nout = out.len();
            for i in 0..nout {
                match samples.next() {
                    Some(s) => out[i] = f32::floor(s * 32768.0f32) as i16,
                    None => {
                        for j in i..nout {
                            out[j] = 0;
                        }
                        playing = false;
                        return;
                    },
                }
            }
        })?;
    }

    stream.stop()?;
    stream.close()?;

    Ok(())
}
