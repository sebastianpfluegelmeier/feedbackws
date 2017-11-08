import("stdfaust.lib");

e  = 2.71828182;
pi = 3.14159265;

// waveshaping functions
// x is the signal input, a and b are parameters
// a and b always are in the range 0 to 1.
// aC and bC are the parameters a and b just scaled to a useful range for the waveshaping functions
sin_fm(x,a,b) = sin(pi * x + aC * sin(bC * pi * x)) with {
    aC = (a ^ 3 * 10) ;
    bC = rint(b * 12);
};

analog_dist(x, a) = (2/(1 + e ^ (-aC * x)) - 1) / (2 / (1 + e ^ (-aC)) - aC) with {
    aC = 1 + a * 19.0;
};

sin_sqrt(x, a, b) = sin(aC * (x + 1) ^ (1/bC) - aC) with {
    aC = 3 + a * 97;
    bC = 2 + b * 7;
};

// three oscillators which are summed together and multiplied by their fader values
oscillators(freq) = sine_fader * os.osc(freq) 
                  + saw_fader * os.sawtooth(freq)
                  + tri_fader * os.triangle(freq);

// the three waveshaping functions summed together and multiplied by their fader values
waveshaper(x, a, b) = sin_fm_fader * sin_fm(x, a, b)
                    + analog_dist_fader * analog_dist(x, a)
                    + sin_sqrt_fader * sin_sqrt(x, a, b);

// faders for the waveshaping input parameters
// "a" and "b" are the labels, 0, 0, 1, 0.001 mean 
// default value, smallest value, largest value, step size
a_fader = hslider("a", 0, 0, 1, 0.001);
b_fader = hslider("b", 0, 0, 1, 0.001);

// oscillator volume faders
sine_fader = hslider("sine", 0, 0, 1, 0.001);
saw_fader  = hslider("saw", 0, 0, 1, 0.001);
tri_fader  = hslider("tri", 0, 0, 1, 0.001);

// waveshaping function volume faders
sin_fm_fader = hslider("sin_fm", 0, 0, 1, 0.001);
analog_dist_fader = hslider("analog_dist", 0, 0, 1, 0.001);
sin_sqrt_fader = hslider("sin_sqrt", 0, 0, 1, 0.001);


// volume, frequency and gain faders
vol_fader  = hslider("vol", 0, 0, 1, 0.001);
freq_fader = hslider("freq", 100, 10, 1000, 0.001);
gain_fader = hslider("gain", 0, 0, 1, 0.001);

// main function
process = (waveshaper(gain_fader * oscillators(freq_fader), a_fader, b_fader)) * vol_fader;
