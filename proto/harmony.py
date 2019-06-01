#!/usr/bin/python3
# Copyright (c) 2019 Bart Massey

import numpy as np
import resamp
import wavio

# Combine a sample with a copy shifted up a third and a copy
# shifted down two octaves for a harmonizing effect.

# Get some samples.
samples = wavio.readwav("../a4.wav")
nsamples = len(samples)

# We will skip through the samples ratio faster.
def make_harmony(ratio):
    harmony = np.array([0] * nsamples, dtype=np.float)
    cutoff = 24000 * min(1, ratio)
    for i in range(nsamples):
        x = (i * ratio) % nsamples
        harmony[i] = resamp.resamp(x, samples, cutoff, 48000, 16)
    return harmony

# A third is four semitones up from the root.
third = make_harmony(2**(4 / 12))

# Two octaves is 1/4 rate.
octaves_down = make_harmony(0.25)

harmony = (samples + third + octaves_down) / 3

wavio.writewav("test.wav", harmony)
