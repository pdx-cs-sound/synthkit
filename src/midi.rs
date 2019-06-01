// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Synthesizer MIDI input.

use std::io;
use std::sync::mpsc;
use std::error::Error;

use midir::MidiInput;

// XXX The name isn't really sufficient: there
// may be multiple connected devices with the same name. We
// should find out how to use port and connection numbers,
// or have the midi reader post a port for connection instead
// of trying to connect directly.

/// Read and process key events from a MIDI keyboard with the
/// given name.
pub fn read_keys(port_name: &str) -> Result<(), Box<Error>> {
    // Keymap indicating which keys are currently down (true).
    let mut keymap = [false;128];
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
            // Leading bit of message is 1 if MIDI "status": the
            // next three bits say which status message. There are
            // also some 8-bit messages.
            match message[0] & 0xf0 {
                // "Note on" message.
                0x90 => {
                    assert_eq!(message.len(), 3);
                    // Data bytes are key number and velocity.
                    // If velocity is zero, treat as a note off message.
                    if message[2] == 0 {
                        println!("note off: {}", message[1]);
                        keymap[message[1] as usize] = false;
                    } else {
                        println!("note on: {} {}", message[1], message[2]);
                        keymap[message[1] as usize] = true;
                    }
                },
                // "Note off" message.
                0x80 => {
                    assert_eq!(message.len(), 3);
                    // Data bytes are key number and velocity.
                    println!("note off: {} {}", message[1], message[2]);
                    keymap[message[1] as usize] = false;
                },
                0xf0 => {
                    match message[0] & 0x0f {
                        0x0e => return,   // Active Sensing
                        // Other special messages ignored for now.
                        _ => println!("unrecognized special {:02x}",
                                      message[0]),
                    }
                },
                // Other messages ignored for now.
                _ => println!("unrecognized status {:02x}", message[0]),
            }
            // XXX Pressing keys for B5 and C5 together will
            // cause end of reading messages. (Exit synth.)
            if keymap[84] && keymap[83] {
                // Send stop message to outer loop.
                sender.send(()).unwrap();
            }
        },
        (),
    );
    // Wait for stop message to leave.
    let () = receiver.recv()?;
    Ok(())
}
