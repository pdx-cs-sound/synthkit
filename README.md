# synthkit: Rust Synthesizer Components
Copyright (c) 2019 Bart Massey

**This is a work in progress** and is barely useful.

This library crate provides some components for building
music synthesizers in Rust. It is currently focused on
sampling synthesis, but many of the components could be used
in some other kind of synthesizer.

The example `harmony` application plays a sample forever as
three-part harmony: A3 A4 C#4.

The example `samplr` application is a Rust "sampling
synthesizer". That is, given a sound sample, it will play it
at various pitches as keys are pressed on a MIDI keyboard.

## Status

* [x] Basic sampler auto-loop: tries to identify a
  reasonable loop point using correlation.

* [x] Basic interpolation / resampling: Uses a
  linear-interpolation and filtering algorithm by
  Ron Nicholson.

* [x] Primitive autotuning: measures the strongest sample
  frequency within a range of 110-1720Hz.

* [x] Replace the callback interface of `portaudio-rs` with a
  blocking interface.

* [ ] Full support for MIDI key-off messages.

* [ ] Support for MIDI messages other than key on-off.

* [ ] Re-entrant MIDI to support multiple separate MIDI
  sources.

* [ ] Cleaned-up library interface suitable for use in
  programs other than the given examples.

* [ ] Multiple formats and styles of sample (currently hardcoded
  to 48000 sps, 1 channel, 16-bit samples).

* [ ] Reliable high-quality sampler auto-loop.

* [ ] Up-front octave resampling for better interpolation
  accuracy at the expense of memory.

* [ ] Fancy autotuning: deal with harmonics and cover a
  larger range of sample frequencies.

* [ ] Replace the callback interface of `midir` with a
  blocking interface and use a reader thread. (This is hard,
  since it will likely involve rewriting parts of `midir`.)

## Acknowledgments

Thanks to Ron Nicholson for the BASIC code for filtered
linear interpolation which was adapted to become the heart
of the resampler. Thanks to the PDX Rust group for valuable
code feedback and suggestions.

## License

This program is licensed under the "MIT License".  Please
see the file LICENSE in the source distribution of this
software for license terms.
