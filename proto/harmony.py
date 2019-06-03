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

# Minimum and maximum expected fundamental frequency of
# samples in Hz.
f_min = 110
f_max = 1720

# Minimum and maximum periods in samples.
s_max = 48000 // f_min
s_min = 48000 // f_max

# Do an FFT to try to find the period of the signal.
nfft = 2**14
nwin = 4 * s_max
windowed = np.hamming(nwin) * np.array(samples[:nwin])
spectrum = np.abs(np.fft.rfft(windowed, n=nfft))
imax = np.argmax(spectrum)
dc = np.abs(spectrum[0])
amax = np.abs(spectrum[imax])
fmax = np.fft.rfftfreq(nfft, d=1/48000)[imax]
pmax = int(48000 / fmax)
print(dc, amax, fmax, pmax)

# Maximum search for autocorrelator.
ac_samples = 2 * pmax

# Sample length for autocorrelator.
ac_length = ac_samples

# Do an autocorrelation to try to find a good place to
# end the samples so they loop.
cmax = None
umax = None
for t in range(ac_samples):
    u = nsamples - ac_length - t
    st = samples[:ac_length]
    su = samples[u:u+ac_length]
    corr = np.dot(st, su)
    if cmax == None or corr > cmax:
        cmax = corr
        umax = u
print(cmax, nsamples - umax)
samples = samples[:umax + ac_length]
nsamples = len(samples)

# Size of lap window from beginning to end of samples.
lap_samples = 0

# Lap the samples.
for i in range(lap_samples):
    c = i / (lap_samples - 1)
    samples[i] *= 1 - c
    samples[i] += c * samples[nsamples + i - lap_samples - 1]

# Use an interpolation window this size around each sample.
# Window should be odd.
window = 9

# Replicate the samples for 5 seconds.
nreplica = 5 * 48000

# We will skip through the samples ratio faster.
def make_harmony(ratio):
    ratio *= 440 / fmax
    cutoff = 20000 * min(1, ratio)
    harmony = np.array([0] * nreplica, dtype=np.float)
    for i in range(nreplica):
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
