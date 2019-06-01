#!/usr/bin/python3
# Copyright (c) 2019 Bart Massey

# WAV file reader / writer.

import array
import numpy as np
import wave as wav

def readwav(fn):
    """
    Read a single-channel 48KHz 16-bit WAV file
    from file named fn and return the samples
    as floats.
    """

    # Get the signal file.
    with wav.open(fn, 'rb') as wavfile:
        # Channels per frame.
        channels = wavfile.getnchannels()
        assert channels == 1

        # Bytes per sample.
        width = wavfile.getsampwidth()
        assert width == 2

        # Sample rate
        rate = wavfile.getframerate()
        assert rate == 48000

        # Number of frames.
        nframes = wavfile.getnframes()

        # Get the signal.
        wav_bytes = wavfile.readframes(nframes)

        # Unpack the signal.
        samples = np.array(array.array('h', wav_bytes),
                           dtype=np.dtype(np.float)) / 32768.0

    return samples

def writewav(fn, samples):
    """
    Take a numpy.array of float 48KHz samples
    and write the 16-bit single-channel WAV file
    fn. Consumes the samples.
    """

    with wav.open(fn, 'wb') as wavfile:
        wavfile.setnchannels(1)
        wavfile.setsampwidth(2)
        wavfile.setframerate(48000)

        samples *= 32767.0
        wav_bytes = samples.astype(np.dtype(np.int16))
        wavfile.writeframes(wav_bytes)
