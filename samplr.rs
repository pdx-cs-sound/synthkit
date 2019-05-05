use hound;
use std::error::Error;
use std::io::{self, ErrorKind};

fn get_loop() -> Result<Vec<f32>, Box<Error>> {
    // Open and check the file.
    let mut wavfile = hound::WavReader::open("loop.wav")?;
    let ws = wavfile.spec();
    if ws.channels != 1
        || ws.bits_per_sample != 16
        || ws.sample_rate != 48000
    {
        return Err(Box::new(io::Error::from(ErrorKind::InvalidData)));
    }

    // Get the signal.
    let signal = wavfile
        .samples::<i16>()
        .map(|s| s.unwrap() as f32)
        .collect();
    Ok(signal)
}

fn main() {
    let signal = get_loop().unwrap();
    println!("{}", signal.len());
}
