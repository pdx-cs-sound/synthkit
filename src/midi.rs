use std::io;
use std::sync::mpsc;
use std::error::Error;

use midir::MidiInput;

pub fn read_keys(port_name: &str) -> Result<(), Box<Error>> {
    let mut keymap = [false;128];
    let (sender, receiver) = mpsc::channel();

    let input = MidiInput::new("samplr")?;
    let inport = (0..input.port_count())
        .find(|p| {
            let name = input.port_name(*p).unwrap();
            name == port_name
        })
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    let _handler = input.connect(
        inport,
        "samplr-input",
        move |_, message: &[u8], _| {
            match message[0] & 0xf0 {
                0x90 => {
                    assert_eq!(message.len(), 3);
                    println!("note on: {} {}", message[1], message[2]);
                    keymap[message[1] as usize] = true;
                },
                0x80 => {
                    assert_eq!(message.len(), 3);
                    println!("note off: {} {}", message[1], message[2]);
                    keymap[message[1] as usize] = false;
                },
                _ => println!("unrecognized"),
            }
            if keymap[84] && keymap[83] {
                sender.send(()).unwrap();
            }
        },
        (),
    );
    let _ = receiver.recv()?;
    Ok(())
}
