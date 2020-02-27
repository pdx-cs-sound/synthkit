// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use std::error::Error;
use std::io::{self, ErrorKind};
use std::sync::{Arc, Mutex};

use cpal::*;
use cpal::traits::*;

use crate::*;

use lazy_static::*;

lazy_static! {
    static ref PLAYER: Arc<Mutex<Option<Player>>> =
        Arc::new(Mutex::new(None));
}

struct Player {
    event_loop: cpal::EventLoop,
    device: cpal::Device,
    format: cpal::Format,
}

/// Gather samples and post for playback.
pub fn play(samples: Stream) -> Result<(), Box<dyn Error>> {

    // Create and initialize cpal state.
    {
        let mut pp = PLAYER.lock()?;
        if pp.is_none() {
            let host = cpal::default_host();
            let device = host.default_output_device()
                .ok_or_else(|| Box::new(io::Error::from(
                    ErrorKind::ConnectionRefused)))?;
            let event_loop = cpal::EventLoop::new();
            let target_rate = cpal::SampleRate(SAMPLE_RATE as u32);
            let format = cpal::StreamConfig {
                channels: 1,
                sample_rate: target_rate,
                data_type: cpal::SampleFormat::I16,
            };
            let player = Player {
                device,
                format,
                event_loop,
            };
            *pp = Some(player);
        }
    }
    let player = PLAYER.lock()?;
    let player = match &*player {
        Some(p) => p,
        None => panic!("internal error: no player"),
    };

    let stream = (&player.event_loop)
        .build_output_stream(&player.device, &player.format)?;
    (&player.event_loop).play_stream(stream);

    let samples = &mut std::sync::Arc::new(std::sync::Mutex::new(samples));
    let (send, recv) = std::sync::mpsc::channel();
    crossbeam::scope(move |scope| {
        let samples = samples.clone();
        let send = send.clone();
        scope.spawn(move || {
            (&player.event_loop).run(move |stream, data| {
                use cpal::UnknownTypeOutputBuffer::I16 as UTOB;
                use cpal::StreamData::Output as SDO;
                if let SDO { buffer: UTOB(mut out) } = data {
                    let out = &mut *out;
                    let nout = out.len();
                    // println!("run {}", nout);
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
                                (&player.event_loop).destroy_stream(stream);
                                send.send(()).unwrap();
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
