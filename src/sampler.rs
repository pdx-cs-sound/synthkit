// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Time stretching and pitch shifting of an audio sample.

#![allow(unused)]

use std::error::Error;
use std::f32::consts::PI;

use dsp::{fft::*, signals::*, windows::*};
use num_complex::*;

use crate::*;

// Minimum and maximum expected fundamental frequency of
// samples in Hz.
const F_MIN: u64 = 110;
const F_MAX: u64 = 1720;

// Minimum and maximum periods in samples.
const S_MAX: u64 = 48000 / F_MIN;
const S_MIN: u64 = 48000 / F_MAX;

// FFT Length. Ideally power of two.
const NFFT: usize = 16_384;

fn max_freq(buf: &[f32]) -> f32 {
    // Create a new Signal with a windowed copy of the buffer
    // maybe zero-padded.
    let mut signal: Vec<Complex64> = buf
        .iter()
        .map(|&s| Complex64::new(f64::from(s), 0.0))
        .collect();
    let init_size = usize::min(signal.len(), NFFT);
    let signal = Signal::from_samples(signal, SAMPLE_RATE as usize);
    let window = hamming(init_size);
    let signal = window.apply(&signal);
    let mut signal = signal.to_vec();
    signal.resize(NFFT, Complex64::new(0.0, 0.0));
    let signal = Signal::from_samples(signal, SAMPLE_RATE as usize);
    let mut fft = ForwardFFT::new(NFFT);
    let f_max = fft.process(&signal).max_freq();
    f_max as f32
}

#[test]
fn test_max_freq_0() {
    assert_eq!(0.0, max_freq(&[1.0; NFFT]));
}

#[test]
fn test_max_freq_nyquist() {
    let buf: Vec<f32> = (0..NFFT)
        .map(|i| if i % 2 == 0 { -1.0 } else { 1.0 })
        .collect();
    assert_eq!(SAMPLE_RATE as f32 / 2.0, max_freq(&buf));
}

fn dot(buf1: &[f32], buf2: &[f32]) -> f32 {
    buf1.iter().zip(buf2.iter()).map(|(s1, s2)| s1 * s2).sum()
}

#[test]
fn test_dot() {
    assert_eq!(14.0, dot(&[1.0, 3.0], &[2.0, 4.0]));
}

// Given a buffer, a corr length, and a range of ending
// lags, return the amount to clip off the end of the
// buffer to get best circular correlation, and the score
// of that clip.
fn best_loop(buf: &[f32], len: usize, lag: std::ops::Range<usize>) -> (f32, usize) {
    let nbuf = buf.len();
    let mut cinfo: Option<(f32, usize)> = None;
    for t in lag {
        let u = nbuf - len - t;
        let corr = dot(&buf[..len], &buf[u..u + len]);
        if cinfo.is_none() || corr > cinfo.unwrap().0 {
            cinfo = Some((corr, u))
        }
    }
    cinfo.unwrap()
}

#[test]
fn test_best_loop() {
    let samples = [1.0, 1.0, 1.0, 0.0];
    assert_eq!(1, best_loop(&samples, 2, 0..2).1);
}

pub fn sampler(buf: &[f32]) -> impl Iterator<f32> {
    // Find the dominant frequency.
    let f_max = max_freq(buf);
    let p_max = f32::floor(SAMPLE_RATE as f32 / f_max + 0.5) as usize;

    // Find the best place to close off the loop and do so.
    let (_, t) = best_loop(buf, 2 * p_max, 0..2 * p_max);
    let buf = &buf[0..buf.len() - t];

    unimplemented!("sampler")
}

// Rust reimplementation of http://www.nicholson.com/rhn/dsp.html#3
// BSD Licensed per author.
// Please see comment at end of file for original source and
// licensing information.
fn resamp(x: f32, indat: &[f32], fmax: f32, wnwdth: i64) -> f32 {
    let alim = indat.len();
    // Calc gain correction factor.
    let r_g = 2.0 * fmax / SAMPLE_RATE as f32;
    let mut r_y = 0.0;
    for i in -wnwdth / 2..wnwdth / 2 - 1 {
        // Calc input sample index.
        let j = (x + i as f32).floor();
        let r_w = 0.5 - 0.5 * f32::cos(2.0 * PI * (0.5 + (j - x) / wnwdth as f32));
        let r_a = 2.0 * PI * (j - x) * fmax / SAMPLE_RATE as f32;
        let r_snc = if j - x == 0.0 {
            1.0
        } else {
            f32::sin(r_a) / r_a
        };
        if j >= 0.0 && (j as usize) < alim {
            r_y += r_g * r_w * r_snc * indat[j as usize];
        }
    }
    r_y
}

// rem - QDSS Windowed-Sinc ReSampling subroutine in Basic
// rem
// rem - This function can also be used for interpolation of FFT results
// rem
// rem function parameters
// rem : x      = new sample point location (relative to old indexes)
// rem            (e.g. every other integer for 0.5x decimation)
// rem : indat  = original data array
// rem : alim   = size of data array
// rem : fmax   = low pass filter cutoff frequency
// rem : fsr    = sample rate
// rem : wnwdth = width of windowed Sinc used as the low pass filter
// rem
// rem resamp() returns a filtered new sample point
// 
// sub resamp(x, indat, alim, fmax, fsr, wnwdth)
//   local i,j, r_g,r_w,r_a,r_snc,r_y	: rem some local variables
//   r_g = 2 * fmax / fsr           : rem Calc gain correction factor
//   r_y = 0
//   for i = -wnwdth/2 to (wnwdth/2)-1 : rem For 1 window width
//     j       = int(x + i)          : rem Calc input sample index
//         : rem calculate von Hann Window. Scale and calculate Sinc
//     r_w     = 0.5 - 0.5 * cos(2*pi*(0.5 + (j - x)/wnwdth))
//     r_a     = 2*pi*(j - x)*fmax/fsr
//     r_snc   = 1  : if (r_a <> 0) then r_snc = sin(r_a)/r_a
//     if (j >= 0) and (j < alim) then
//       r_y   = r_y + r_g * r_w * r_snc * indat(j)
//     endif
//   next i
//   resamp = r_y                  : rem Return new filtered sample
// end sub
// 
// rem  - Ron Nicholson's QDSS ReSampler cookbook recipe
// rem                 QDSS = Quick, Dirty, Simple and Short
// rem  Version 0.1b - 2007-Aug-01
// rem  Copyright 2007 Ronald H. Nicholson Jr.
// rem  No warranties implied.  Error checking, optimization, and
// rem    quality assessment of the "results" is left as an exercise
// rem    for the student.
// rem  (consider this code Open Source under a BSD style license)
// rem  IMHO. YMMV.  http://www.nicholson.com/rhn/dsp.html
