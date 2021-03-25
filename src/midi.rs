// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer MIDI input.

use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::sync::mpsc;

use midir::MidiInput;
use wmidi::MidiMessage::*;
use wmidi::*;

/// Read and process key events from a MIDI keyboard with the
/// given name.
pub fn read_keys(
    port_name: &str,
) -> Result<mpsc::Receiver<MidiMessage<'static>>, Box<dyn Error>> {
    // Channel for communicating events from midir callback.
    let (sender, receiver) = mpsc::sync_channel(0);

    // Set up for reading key events.
    let input = MidiInput::new("samplr")?;
    let inport = (0..input.port_count())
        .find(|p| {
            let name = input.port_name(*p).unwrap();
            let port_index = name.rfind(' ').unwrap();
            &name[..port_index] == port_name
        })
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    // Read and process key events.
    let handler = input.connect(
        inport,
        "samplr-input",
        move |_, message: &[u8], _| {
            let message = MidiMessage::try_from(message).unwrap();
            match message {
                NoteOn(c, note, velocity) => {
                    let velocity8 = u8::from(velocity);
                    // If velocity is zero, treat as a note off message.
                    if velocity8 == 0 {
                        sender
                            .send(NoteOff(c, note, velocity))
                            .unwrap();
                        println!("note off: {}", note);
                    } else {
                        sender.send(NoteOn(c, note, velocity)).unwrap();
                        println!("note on: {} {}", note, velocity8);
                    }
                }
                NoteOff(c, note, velocity) => {
                    let velocity8 = u8::from(velocity);
                    sender.send(NoteOff(c, note, velocity)).unwrap();
                    println!("note off: {} {}", note, velocity8);
                }
                ActiveSensing => {
                    // Active sensing ignored for now.
                }
                // Other messages ignored for now.
                m => println!("unrecognized message {:?}", m),
            }
        },
        (),
    );
    std::mem::forget(handler?);
    Ok(receiver)
}
