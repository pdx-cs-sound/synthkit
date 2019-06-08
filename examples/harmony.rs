// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Harmonizer demo example using synthkit-rs.
use synthkit::*;

fn main() {
    let wav = std::env::args().nth(1).unwrap();
    let sound = get_sample(&wav).unwrap();
    let sloop = Loop::new(&sound);
    let root = 440.0;
    let third = root * f32::powf(2.0, 4.0 / 12.0);
    let octaves_down = root / 4.0;
    let mixer = Mixer::with_streams(vec![
        Box::new(sloop.iter(root)),
        Box::new(sloop.iter(third)),
        Box::new(sloop.iter(octaves_down)),
    ]);
    let duration = (5.0 * SAMPLE_RATE as f32).floor() as usize;
    let harmony: Vec<f32> = mixer.take(duration).collect();
    play(harmony).unwrap();
}
