// Copyright © 2019 Bart Massey
// [This program is licensed under the "MIT License"]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Time stretching and pitch shifting of an audio sample.

use std::f32::consts::PI;

use dsp::{fft::*, signals::*, windows::*};
use num_complex::*;

use crate::*;

// Width of resampling filter in samples. Should be odd,
// since centered on target sample. Larger is better and
// slower.
const RESAMP_WIDTH: i64 = 9;

// Minimum and maximum expected fundamental frequency of
// samples in Hz.
const F_MIN: f32 = 110.0;
const F_MAX: f32 = 1720.0;

// FFT Length. Ideally power of two.
const NFFT: usize = 16_384;

// Find the maximum frequency of the buffer.
fn max_freq(buf: &[f32]) -> f32 {
    // Create a new Signal.
    let signal: Vec<Complex64> = buf
        .iter()
        .take(NFFT)
        .map(|&s| Complex64::new(f64::from(s), 0.0))
        .collect();
    let init_size = signal.len();
    let signal = Signal::from_samples(signal, SAMPLE_RATE as usize);

    // Window the signal.
    let window = hamming(init_size);
    let signal = window.apply(&signal);

    // Do the FFT and return the maximum frequency.
    let mut signal = signal.to_vec();
    signal.resize(NFFT, Complex64::new(0.0, 0.0));
    let signal = Signal::from_samples(signal, SAMPLE_RATE as usize);
    let mut fft = ForwardFFT::new(NFFT);
    let f_max = fft.process(&signal).max_freq();
    f_max as f32
}

#[test]
// Check that a DC signal has a DC maximum.
fn test_max_freq_0() {
    assert_eq!(0.0, max_freq(&[1.0; NFFT]));
}

#[test]
// Check that a maximum-frequency signal is correct.
fn test_max_freq_nyquist() {
    let buf: Vec<f32> = (0..NFFT)
        .map(|i| if i % 2 == 0 { -1.0 } else { 1.0 })
        .collect();
    assert_eq!(SAMPLE_RATE as f32 / 2.0, max_freq(&buf));
}

// Plain old dot product.
fn dot(buf1: &[f32], buf2: &[f32]) -> f32 {
    buf1.iter().zip(buf2.iter()).map(|(s1, s2)| s1 * s2).sum()
}

#[test]
// Test that some dot product comes out right.
fn test_dot() {
    assert_eq!(14.0, dot(&[1.0, 3.0], &[2.0, 4.0]));
}

// Given a buffer, a corr length, and a range of end
// segments at lag..0, return the amount to clip off the end
// of the buffer to get best circular correlation, and the
// score of that clip.
fn best_loop(
    buf: &[f32],
    len: usize,
    lag: usize,
) -> (f32, usize) {
    let nbuf = buf.len();
    let mut cinfo: Option<(f32, usize)> = None;
    for t in (0..lag).rev() {
        let u = nbuf - len - t;
        let corr = dot(&buf[..len], &buf[u..u + len]);
        if cinfo.is_none() || corr > cinfo.unwrap().0 {
            cinfo = Some((corr, t))
        }
    }
    cinfo.unwrap()
}

#[test]
// Test that the end sample is clipped off a short thing.
fn test_best_loop() {
    let samples = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0];
    assert_eq!(1, best_loop(&samples, 2, 2).1);
}

/// Iterator producing resampled audio samples.  This is an
/// unbounded iterator.
pub struct Samples<'a> {
    buf: &'a [f32],
    incr: f32,
    cutoff: f32,
    x: f32,
}

impl<'a> Samples<'a> {

    // Make a new resampling iterator.
    pub fn new(sloop: &'a Loop, incr: f32, cutoff: f32) -> Self {
        assert!(incr.abs() < RESAMP_WIDTH as f32 / 2.0);
        Self { buf: &sloop.buf, incr, cutoff, x: 0.0 }
    }

    /// Reset the iterator to the beginning of the loop.
    /// This will ensure that it starts at a zero-crossing.
    ///
    /// # Bug
    ///
    /// The above assertion is not yet true: samples don't
    /// necessarily start at zero crossings right now.
    pub fn reset(&mut self) {
        self.x = 0.0;
    }
}

impl<'a> Iterator for Samples<'a> {
    type Item = f32;

    /// Return the next sample from the iterator.
    fn next(&mut self) -> Option<f32> {
        let s = resamp(self.x, self.buf, self.cutoff, RESAMP_WIDTH);
        let nbuf = self.buf.len() as f32;
        self.x += self.incr;
        while self.x >= nbuf {
            self.x -= nbuf;
        }
        Some(s)
    }
}

/// An audio sample loop that has been frequency-analyzed
/// and trimmed for looping.
pub struct Loop {
    buf: Vec<f32>,
    freq: Option<f32>,
}

impl Loop {
    /// Make a `Loop` out of some samples.
    pub fn new(buf: &[f32]) -> Self {
        // Find the dominant frequency.
        let f_max = max_freq(buf);
        let p = |f| f32::floor(SAMPLE_RATE as f32 / f + 0.5) as usize;
        let (freq, p_max) = if f_max >= F_MIN && f_max <= F_MAX {
            (Some(f_max), p(f_max))
        } else {
            (None, p(F_MAX))
        };
         
        // Find the best place to close off the loop and do so.
        let (_, t) = best_loop(buf, 2 * p_max, 2 * p_max);
        let buf = &buf[0..buf.len() - t];

        // Return the loop for future sampling.
        Self { buf: buf.to_owned(), freq }
    }

    /// Iterator over the samples of a loop, resampled
    /// to the given target frequency.
    pub fn iter<'a>(&'a self, freq: f32) -> Samples<'a> {
        let incr = match self.freq {
            Some(f) => freq / f,
            None => 1.0,
        };
        let cutoff = 20_000.0 * f32::min(1.0, incr);
        Samples::new(&self, incr, cutoff)
    }
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
