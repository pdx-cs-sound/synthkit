// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Synthesizer demo example using synthkit-rs.
use synthkit::*;

fn main() {
    // Get a signal from a WAV file.
    let signal = get_sample("loop.wav").unwrap();
    // Play signal on audio output.
    let samples = Box::new(signal.into_iter());
    play(samples).unwrap();
    // Read and decode MIDI keys.
    read_keys("Mobile Keys 49").unwrap();
}
