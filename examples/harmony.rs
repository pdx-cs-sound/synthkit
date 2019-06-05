// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Harmonizer demo example using synthkit-rs.
use synthkit::*;

fn main() {
    let wav = std::env::args().nth(1).unwrap();
    let sound = get_sample(wav).unwrap();
    let loop = Loop::new(&sound);
    let root = 440.0;
    let third = root * f32::pow(2.0, 4.0 / 12);
    let octaves_down = root / 4.0;
    let mut mixed = Mixed::new(vec![
        loop.iter(root),
        loop.iter(third),
        loop.iter(octaves_down),
    ]);
    let duration = (5.0 * SAMPLE_RATE as f32).floor() as usize;
    let harmony: Vec<f32> = mixed.iter().take(duration).collect();
    play(&harmony).unwrap();
}
