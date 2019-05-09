// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use lazy_static::*;
use portaudio as pa;

use std::error::Error;
use std::sync::{Arc, Mutex};

// Data used to play samples on audio output device.
struct Player {
    // Index indicating sample next to be played.
    index: usize,
    // Samples to be played, circularly.
    buf: Vec<f32>,
}

// Set up safe synchronized global access to the player.
lazy_static! {
    static ref PLAYER: Arc<Mutex<Option<Player>>> =
        Arc::new(Mutex::new(None));
}

// Supply portaudio with a requested chunk of samples.
fn player(out: pa::OutputStreamCallbackArgs<i16>)
    -> pa::stream::CallbackResult
{
    let mut pp = PLAYER.lock().unwrap();
    let pl = pp.as_mut().unwrap();
    let nbuf = pl.buf.len();

    for i in 0..out.frames {
        out.buffer[i] = f32::floor(pl.buf[pl.index] * 32768.0f32) as i16;
        pl.index += 1;
        if pl.index >= nbuf {
            pl.index = 0;
        }
    }

    pa::Continue
}

// Post the given samples for loop playback.
pub fn play(buf: Vec<f32>) -> Result<(), Box<Error>> {
    // Install a new player.
    {
        let mut plp = PLAYER.lock()?;
        *plp = Some(Player{ index: 0, buf });
    }

    // Create and initialize audio output.
    let out = pa::PortAudio::new()?;
    let mut settings =
        out.default_output_stream_settings(
            1,   // 1 channel.
            crate::SAMPLE_RATE as f64,
            0_u32,   // Least possible buffer.
        )?;
    settings.flags = pa::stream_flags::CLIP_OFF;
    let mut stream =
        out.open_non_blocking_stream(settings, player)?;
    
    // Play 1s of samples and then stop everything.
    stream.start()?;
    out.sleep(1000);
    stream.stop()?;
    stream.close()?;

    Ok(())
}
