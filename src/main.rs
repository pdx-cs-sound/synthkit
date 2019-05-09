// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

mod sample;
mod play;
mod midi;

use sample::*;
use play::*;
use midi::*;

const SAMPLE_RATE: u32 = 48_000;

fn main() {
    // Get a signal from a WAV file.
    let signal = get_sample().unwrap();
    // Play signal on audio output.
    play(signal).unwrap();
    // Read and decode MIDI keys.
    read_keys("Mobile Keys 49").unwrap();
}
