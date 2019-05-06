use lazy_static::*;
use portaudio as pa;

use std::error::Error;
use std::sync::{Arc, Mutex};

struct Player {
    index: usize,
    buf: Vec<f32>,
}

lazy_static! {
    static ref PLAYER: Arc<Mutex<Option<Player>>> =
        Arc::new(Mutex::new(None));
}

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

pub fn play(buf: Vec<f32>) -> Result<(), Box<Error>> {
    {
        let mut plp = PLAYER.lock()?;
        *plp = Some(Player{ index: 0, buf });
    }

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
    stream.start()?;
    out.sleep(1000);
    stream.stop()?;
    stream.close()?;

    Ok(())
}
