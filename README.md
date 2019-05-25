# synthkit: Rust Synthesizer Components
Copyright (c) 2019 Bart Massey

**This is a work in progress** and does not do anything
useful yet.

This crate provides some components for building music
synthesizers in Rust.

The example `samplr` application is a Rust "sampling
synthesizer". That is, given a sound sample, it will play it
at various pitches as keys are pressed on a MIDI keyboard.

## Features

It is easier to list the features *not intended* for the
MVP:

* Multiple formats and styles of sample: currently hardcoded
  to 48Ksps, 1 channel, 16-bit samples.

* Sampler:

  * Correct loops: initial plan is to hardcode a loop interval
    at the end of the sample and interpolate in. This will
    sound terrible.

  * Good interpolation / resampling: initial plan is strictly
    linear with no filtering.

  * Autotuning: the intended sample frequency will be
    hardcoded to start.

* MIDI:

  * Support for MIDI messages other than key on-off.

  * Replace the callback interface of `midir` with a
    blocking interface and use a reader thread. This is
    hard, since it will likely involve rewriting parts of
    `midir`.

* Player:

  * Replace the callback interface of `portaudio-rs` with a
    blocking interface (like the C version of portaudio has)
    and use a player thread. This is hard, since it will
    likely involve rewriting parts of `portaudio-rs`.

So basically, the first working version will be the most
minimal thing that can enable making a plausible and vaguely
interesting noise.

## License

This program is licensed under the "MIT License".  Please
see the file LICENSE in the source distribution of this
software for license terms.
