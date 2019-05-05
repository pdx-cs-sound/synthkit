mod sample;

use sample::*;

fn main() {
    let signal = get_loop().unwrap();
    println!("{}", signal.len());
}
