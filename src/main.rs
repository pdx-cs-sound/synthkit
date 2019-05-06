mod sample;
mod play;
mod midi;

use sample::*;
use play::*;
use midi::*;

const SAMPLE_RATE: u32 = 48_000;

fn main() {
    let signal = get_sample().unwrap();
    play(signal).unwrap();
    read_keys("Mobile Keys 49 32:0").unwrap();
}
