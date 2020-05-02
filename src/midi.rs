// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer MIDI input.

use std::convert::TryFrom;
use std::error::Error;
use std::io;
use std::sync::{mpsc, Mutex};

use midir::{MidiInput, MidiInputConnection};
use once_cell::sync::OnceCell;
use wmidi::*;
use wmidi::MidiMessage::*;

static HANDLER: OnceCell<Mutex<MidiInputConnection<()>>> = OnceCell::new();

/// Read and process key events from a MIDI keyboard with the
/// given name.
pub fn read_keys(port_name: &str) -> Result<mpsc::Receiver<MidiMessage<'static>>, Box<dyn Error>> {
    // Keymap indicating which keys are currently down (true).
    let mut keymap = [false; 128];
    // Channel for communicating events from midir callback.
    let (sender, receiver) = mpsc::channel();

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
                        println!("note off: {}", note);
                        keymap[note as usize] = false;
                        sender.send(NoteOff(c, note, velocity)).unwrap();
                    } else {
                        println!("note on: {} {}", note, velocity8);
                        keymap[note as usize] = true;
                        sender.send(NoteOn(c, note, velocity)).unwrap();
                    }
                },
                NoteOff(c, note, velocity) => {
                    let velocity8 = u8::from(velocity);
                    println!("note off: {} {}", note, velocity8);
                    keymap[note as usize] = false;
                    sender.send(NoteOff(c, note, velocity)).unwrap();
                },
                ActiveSensing => {
                    // Active sensing ignored for now.
                },
                // Other messages ignored for now.
                m => println!("unrecognized message {:?}", m),
            }
        },
        (),
    );
    HANDLER.set(Mutex::new(handler?))
        .unwrap_or_else(|_| panic!("cannot set handler"));
    Ok(receiver)
}
