// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer MIDI input.

use std::error::Error;
use std::io;
use std::sync::mpsc;

use midir::MidiInput;
use wmidi::*;
use wmidi::MidiMessage::*;

/// Read and process key events from a MIDI keyboard with the
/// given name.
pub fn read_keys(port_name: &str) -> Result<(), Box<dyn Error>> {
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
    let _handler = input.connect(
        inport,
        "samplr-input",
        move |_, message: &[u8], _| {
            let message = MidiMessage::from_bytes(message).unwrap();
            match message {
                NoteOn(c, note, velocity) => {
                    // If velocity is zero, treat as a note off message.
                    if velocity == 0 {
                        println!("note off: {}", note);
                        keymap[note as usize] = false;
                        sender.send(NoteOff(c, note, velocity)).unwrap();
                    } else {
                        println!("note on: {} {}", note, velocity);
                        keymap[note as usize] = true;
                        sender.send(NoteOn(c, note, velocity)).unwrap();
                    }
                },
                NoteOff(c, note, velocity) => {
                    println!("note off: {} {}", note, velocity);
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
    // Wait for stop message to leave.
    loop {
        let _ = receiver.recv()?;
    }
    #[allow(unused)]
    Ok(())
}
