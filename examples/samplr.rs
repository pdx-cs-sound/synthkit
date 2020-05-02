// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Synthesizer demo example using synthkit-rs.

use std::borrow::BorrowMut;
use std::sync::Mutex;
use std::thread;

use once_cell::sync::OnceCell;
use wmidi::MidiMessage::*;

use synthkit::*;

static MIXER: OnceCell<Mutex<Mixer<'static>>> = OnceCell::new();
static SLOOP: OnceCell<Loop> = OnceCell::new();

fn main() {
    // Parse arguments.
    let args: Vec<String> = std::env::args().collect();
    let wav = &args[1];
    let kbd = &args[2];

    // Get a signal from a WAV file, make a loop,
    // set up the mixer.
    let sound = get_sample(wav).unwrap();
    SLOOP.set(Loop::new(&sound)).unwrap();
    MIXER.set(Mutex::new(Mixer::new())).unwrap();

    // Start the keyreader to get input.
    let keystream = read_keys(kbd).unwrap();
    // Start outputting samples.
    let player = thread::spawn(|| {
        play(MIXER.get().unwrap()).unwrap();
    });
    for kev in keystream {
        match kev {
            NoteOn(_c, note, _vel) => {
                let gsloop = SLOOP.get().unwrap();
                let mut gmixer = MIXER.get().unwrap().lock().unwrap();
                let samples = gsloop.iter_freq(note.to_freq_f32());
                gmixer.borrow_mut().add(samples.clone());
            }
            NoteOff(_c, _note, _vel) => {
                let mut gmixer = MIXER.get().unwrap().lock().unwrap();
                gmixer.clear()
            }
            _ => (),
        }
    }
    player.join().unwrap();
}
