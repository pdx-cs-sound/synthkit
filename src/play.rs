// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use portaudio as pa;

use std::error::Error;
use std::sync::{Arc, Mutex};

/// Data used to play samples on audio output device.
struct Player {
    /// Index indicating sample next to be played.
    index: usize,
    /// Samples to be played, circularly.
    buf: Vec<f32>,
}

/// Post the given buffer of normalized float samples for
/// loop playback.
pub fn play(buf: Vec<f32>) -> Result<(), Box<Error>> {
    // Set up the player.
    let player =
        Arc::new(Mutex::new(Player { index: 0, buf }));

    // Callback supplies portaudio with a requested chunk of samples.
    let playback = move |out: pa::OutputStreamCallbackArgs<i16>|
                   -> pa::stream::CallbackResult {

        // Borrow the sample buffer from the player safely.
        let mut pl = player.lock().unwrap();
        let nbuf = pl.buf.len();

        // Copy the requested samples into the output buffer,
        // converting as we go.
        for i in 0..out.frames {
            out.buffer[i] = f32::floor(pl.buf[pl.index] * 32768.0f32) as i16;
            pl.index += 1;
            if pl.index >= nbuf {
                pl.index = 0;
            }
        }

        // Keep the stream going.
        pa::Continue
    };

    // Create and initialize audio output.
    let out = pa::PortAudio::new()?;
    let mut settings = out.default_output_stream_settings(
        1, // 1 channel.
        crate::SAMPLE_RATE as f64,
        0_u32, // Least possible buffer.
    )?;
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut stream = out.open_non_blocking_stream(settings, playback)?;

    // Play 1s of samples and then stop everything.
    stream.start()?;
    out.sleep(5000);
    stream.stop()?;
    stream.close()?;

    Ok(())
}
