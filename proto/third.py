#!/usr/bin/python3
# Copyright (c) 2019 Bart Massey

import numpy as np
import resamp
import wavio

# Combine a sample with a copy of itself
# shifted up a third for a harmonizing effect.

# Get some samples.
samples = wavio.readwav("../a4.wav")
nsamples = len(samples)

# A third is four semitones up from the root.
# We will skip through the samples this much faster.
third = 2**(4 / 12)

harmony = np.array([0] * nsamples, dtype=np.float)
for i in range(nsamples):
    x = (i * third) % nsamples
    harmony[i] = resamp.resamp(x, samples, 24000, 48000, 16)

harmony = (harmony + samples) / 2

wavio.writewav("test.wav", harmony)
