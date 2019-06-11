// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use std::error::Error;
use std::io::{self, ErrorKind};

use crate::*;

/// Number of samples for a blocking write.
const OUT_FRAMES: usize = 12;

/// Gather samples and post for playback.
pub fn play(mut samples: Stream) -> Result<(), Box<Error>> {

    // Create and initialize audio output.
    let event_loop = cpal::EventLoop::new();
    let device = cpal::default_output_device()
        .ok_or_else(||
            Box::new(io::Error::from(ErrorKind::ConnectionRefused)))?;
    let target_rate = cpal::SampleRate(SAMPLE_RATE as u32);
    let has_format = device
        .supported_output_formats()?
        .find(|&f| {
            f.channels == 1 &&
                f.min_sample_rate <= target_rate &&
                f.max_sample_rate >= target_rate &&
                f.data_type == cpal::SampleFormat::I16
        })
        .is_some();
    if !has_format {
        return Err(Box::new(cpal::FormatsEnumerationError::InvalidArgument));
    }
    let format = cpal::Format {
        channels: 1,
        sample_rate: target_rate,
        data_type: cpal::SampleFormat::I16,
    };
    let stream = event_loop.build_output_stream(&device, &format)?;
    event_loop.play_stream(stream);

    let samples = &mut std::sync::Arc::new(std::sync::Mutex::new(samples));
    event_loop.run(|_stream, data| {
        use cpal::UnknownTypeOutputBuffer::I16 as UTOB;
        use cpal::StreamData::Output as SDO;
        if let SDO { buffer: UTOB(mut out) } = data {
            let nout = out.len();
            for i in 0..nout {
                match samples.lock().unwrap().next() {
                    Some(s) => out[i] = f32::floor(s * 32768.0f32) as i16,
                    None => std::process::exit(0),
                }
            }
        } else {
            panic!("unexpected output buffer type");
        }
    });

    // Should never actually be able to get here.
    #[allow(unused)]
    Err(Box::new(io::Error::from(ErrorKind::Interrupted)))
}
