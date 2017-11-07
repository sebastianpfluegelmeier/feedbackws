import("stdfaust.lib");

e  = 2.71828182;
pi = 3.14159265;
input_curve = 5.0;

input_logis(x) = (a - b) / (c - b) with {
    a = 1/(1 + e ^ (-input_curve * (x - 0.5)));
    b = 1/(1 + e ^ (-input_curve * -0.5));
    c = 1/(1 + e ^ (-input_curve * 0.5));
};

sin_fm(x,a,b) = sin(pi * x + aC * sin(bC * pi * x)) with {
    aC = input_logis(10 * a);
    bC = rint(b * 12);
};

analog_dist(x, a) = (2/(1 + e ^ (-aC * x)) - 1) / (2 / (1 + e ^ (-aC)) - aC) with {
    aC = 1 + (input_logis(a)*19.0);
};

sin_sqrt(x, a, b) = sin(aC * (x + 1) ^ (1/bC) - aC) with {
    aC = 3 + input_logis(a) * 97;
    bC = 2 + input_logis(b) * 7;
};

osc(freq) = sine * os.osc(freq) 
          + saw * os.sawtooth(freq)
          + tri * os.triangle(freq);

waveshaper(x, a, b) = sin_fm_fader * sin_fm(x, a, b)
                      + analog_dist_fader * analog_dist(x, a)
                      + sin_sqrt_fader * sin_sqrt(x, a, b);

a_fader = hslider("6 a", 0, 0, 1, 0.01);
b_fader = hslider("7 b", 0, 0, 1, 0.01);
sine = hslider("3 sine", 0, 0, 1, 0.01);
saw  = hslider("4 saw", 0, 0, 1, 0.01);
tri  = hslider("5 tri", 0, 0, 1, 0.01);
sin_fm_fader      = hslider("0 sin_fm", 0, 0, 1, 0.01);
analog_dist_fader = hslider("1 analog_dist", 0, 0, 1, 0.01);
sin_sqrt_fader    = hslider("2 sin_sqrt", 0, 0, 1, 0.01);
vol_fader  = hslider("vol", 0, 0, 1, 0.01);
freq_fader  = hslider("freq", 100, 10, 10000, 0.01);
gain_fader = hslider("gain", 0, 0, 10, 0.01);

process = waveshaper(gain_fader * osc(freq_fader), a_fader, b_fader) * vol_fader;
