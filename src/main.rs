mod sample;
mod play;

use sample::*;
use play::*;

const SAMPLE_RATE: u32 = 48_000;

fn main() {
    let signal = get_sample().unwrap();
    play(signal).unwrap();
}
