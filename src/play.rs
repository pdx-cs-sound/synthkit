// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer audio player.

use std::error::Error;
use std::io::{self, ErrorKind};
use std::sync::{Arc, Mutex};

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
    let mut player = PLAYER.lock()?;
    if player.is_none() {
        let host = cpal::default_host();
        let event_loop = host.event_loop();
        let device = host.default_output_device()
            .ok_or_else(|| Box::new(io::Error::from(
                ErrorKind::ConnectionRefused)))?;
        let target_rate = cpal::SampleRate(SAMPLE_RATE as u32);
        let format = cpal::Format {
            channels: 1,
            sample_rate: target_rate,
            data_type: cpal::SampleFormat::I16,
        };
        *player = Some(Player { event_loop, device, format });
    }
    let player = match &*player {
        Some(p) => p,
        None => panic!("internal error: no player"),
    };

    let stream_id = (&player.event_loop)
        .build_output_stream(&player.device, &player.format)?;
    (&player.event_loop).play_stream(stream_id)?;

    let samples = &mut std::sync::Arc::new(std::sync::Mutex::new(samples));
    let (send, recv) = std::sync::mpsc::channel();
    crossbeam::scope(move |scope| {
        let samples = samples.clone();
        let send = send.clone();
        scope.spawn(move || {
            (&player.event_loop).run(move |stream_id, data| {
                use cpal::UnknownTypeOutputBuffer::I16 as UTOB;
                use cpal::StreamData::Output as SDO;
                if let SDO { buffer: UTOB(mut out) } = data.unwrap() {
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
                                (&player.event_loop).destroy_stream(stream_id);
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
