// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use std::error::Error;
use std::io::{self, ErrorKind};

use crate::*;

/// Gather samples and post for playback.
pub fn play(samples: Stream) -> Result<(), Box<Error>> {

    // Create and initialize audio output.
    let event_loop = cpal::EventLoop::new();
    let device = cpal::default_output_device()
        .ok_or_else(||
            Box::new(io::Error::from(ErrorKind::ConnectionRefused)))?;
    let target_rate = cpal::SampleRate(SAMPLE_RATE as u32);
    let has_format = device
        .supported_output_formats()?
        .find(|f| {
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
    let (send, recv) = std::sync::mpsc::channel();
    crossbeam::scope(|scope| {
        let samples = samples.clone();
        let send = send.clone();
        let event_loop = &event_loop;
        scope.spawn(move || {
            event_loop.run(move |stream, data| {
                use cpal::UnknownTypeOutputBuffer::I16 as UTOB;
                use cpal::StreamData::Output as SDO;
                if let SDO { buffer: UTOB(mut out) } = data {
                    let out = &mut *out;
                    let nout = out.len();
                    println!("run {}", nout);
                    let mut samples = samples.lock().unwrap();
                    for i in 0..nout {
                        match samples.next() {
                            Some(s) => {
                                out[i] =
                                    f32::floor(s * 32768.0) as i16;
                            },
                            None => {
                                for j in i..nout {
                                    out[j] = 0;
                                }
                                send.send(()).unwrap();
                                event_loop.destroy_stream(stream);
                                break;
                            },
                        }
                    }
                } else {
                    panic!("unexpected output buffer type");
                }
            });
        });
        println!("spawned");
        let () = recv.recv().unwrap();
        println!("done");
    });

    Ok(())
}
