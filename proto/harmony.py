#!/usr/bin/python3
# Copyright (c) 2019 Bart Massey

import numpy as np
import resamp
import wavio

# Combine a sample with a copy shifted up a third and a copy
# shifted down two octaves for a harmonizing effect.
# Play it for 5 seconds.

# Get some samples.
samples = wavio.readwav("../loop.wav")
nsamples = len(samples)

# Maximum search for autocorrelator.
ac_msecs = 200
ac_samples = ac_msecs * 48000 // 1000

# Sample length for autocorrelator.
ac_length = ac_samples // 8

# Do an autocorrelation to try to find a good place to
# end the samples so they loop.
cmax = None
umax = None
for t in range(ac_samples):
    u = nsamples - ac_samples - ac_length + t
    st = samples[:ac_length]
    su = samples[u:u+ac_length]
    corr = np.dot(st, su)
    if cmax == None or corr > cmax:
        cmax = corr
        umax = u
samples = samples[:umax + ac_length]
nsamples = len(samples)

# Milliseconds of lap window at beginning and end of harmony.
lap_msecs = 100
lap_samples = lap_msecs * 48000 // 1000

# Lap the samples.
for i in range(lap_samples):
    c = i / (lap_samples - 1)
    samples[i] *= 1 - c
    samples[i] += c * samples[nsamples + i - lap_samples - 1]

# Replicate the harmony for 5 seconds.
nreplica = 5 * 48000
replica = np.array([0]*nreplica, dtype=np.float)
for i in range(nreplica):
    replica[i] = samples[i % nsamples]
wavio.writewav("ac.wav", replica)

# Use an interpolation window this size around each sample.
# Window should be odd.
window = 65

# We will skip through the samples ratio faster.
def make_harmony(ratio):
    cutoff = 24000 * min(1, ratio)
    harmony = np.array([0] * nsamples, dtype=np.float)
    for i in range(nsamples):
        x = (i * ratio) % nsamples
        harmony[i] = resamp.resamp(x, samples, cutoff, 48000, window)
    return harmony

# Make a slightly truncated copy of the root.
root = make_harmony(1)

# A third is four semitones up from the root.
third = make_harmony(2**(4 / 12))

# Two octaves is 1/4 rate.
octaves_down = make_harmony(0.25)

# Mix the notes.
harmony = (root + third + octaves_down) / 3
nharmony = len(harmony)

# Replicate the harmony for 5 seconds.
nreplica = 5 * 48000
replica = np.array([0]*nreplica, dtype=np.float)
for i in range(nreplica):
    replica[i] = harmony[i % nharmony]

wavio.writewav("harmony.wav", replica)
