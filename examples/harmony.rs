// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

// Harmonizer demo example using synthkit-rs.

use std::sync::Mutex;

use synthkit::*;

fn main() {
    let wav = std::env::args().nth(1).unwrap();
    let sound = get_sample(&wav).unwrap();
    let sloop = Box::leak(Box::new(Loop::new(&sound)));
    let root = 440.0;
    let third = root * f32::powf(2.0, 4.0 / 12.0);
    let octaves_down = root / 4.0;
    let mixer = Box::leak(Box::new(Mutex::new(Mixer::with_streams(vec![
        (69, sloop.iter_freq(root)),         // A4
        (73, sloop.iter_freq(third)),        //C#5
        (45, sloop.iter_freq(octaves_down)), // A2
    ]))));
    let player = play(&*mixer).unwrap();
    player.block();
}
