#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin, HostCallback};
use vst2::buffer::AudioBuffer;
use std::f32::consts::PI;
use std::f32::consts::E;

const INPUT_CURVE: f32 =  5.0;

/// Array of (waveshape function, descriptor string) tuples.
static FUNCTIONS: &'static [(fn(f32, f32, f32, f32, f32) -> f32, &str)] = &[
    (sin_fm, "sin(pi * x + a * sin(b * pi * x))"),
    (analog_dist, "(2/(1 + E ^ (-a * x)) - 1) / (2/(1 + E ^ (-a) - 1)"),
    (sin_sqrt, "sin(a * (x + 1) ^ (1 / b) - a)")
];

// All waveshape functions
fn sin_fm(x: f32, mut a: f32, mut b:f32, _:f32, _:f32) -> f32 {
    a = input_logis(10.0 * a);
    b = (b * 12.0).round();
    f32::sin(PI * x + a * f32::sin(b * PI * x))
}

fn analog_dist(x: f32, mut a: f32, _: f32, _: f32, _:f32) -> f32 {
    a = 1.0 + (input_logis(a)*19.0);
    (2.0/(1.0+E.powf(-a*x)) - 1.0)/(2.0/(1.0+E.powf(-a)) - 1.0)
}

fn sin_sqrt(x: f32, mut a: f32, mut b:f32, _:f32, _:f32) -> f32 {
    a = 3.0 + input_logis(a) * 97.0;
    b = 2.0 + input_logis(b) * 7.0;
    f32::sin(a * (x + 1.0).powf(1.0/b) - a)
}

// input curve function
fn input_logis(x: f32) -> f32 {
    let a = 1.0/(1.0 + E.powf(-INPUT_CURVE * (x - 0.5)));
    let b = 1.0/(1.0 + E.powf(-INPUT_CURVE * -0.5));
    let c = 1.0/(1.0 + E.powf(-INPUT_CURVE * 0.5));
    (a - b)/(c - b)
}

/// Main Plugin Struct
#[derive(Default)]
struct FeedbackWS {
    // last output samples for feedback stuff
    last_sample_l: f32,
    last_sample_r: f32,

    // input parameters
    feedback: f32,
    parameter_a: f32,
    parameter_b: f32,
    parameter_c: f32,
    parameter_d: f32,
    gain: f32,
    stereo_depth: f32,
    stereo_color: f32,
    beta: f32,
    current_function: usize,
    // length of FUNCTIONS array
    functions_len: usize,
}

impl Plugin for FeedbackWS {
    fn get_info(&self) -> Info {
        Info {
            name: "FeedbackWS".to_string(),
            unique_id: 543229834,
            inputs: 2,
            outputs: 2,
            parameters: 10,
            ..Default::default()
        }
    }

    /// main processing function, gets audio buffers and writes into the output buffers.
    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();
        // iterate over channels
        for (chan_i, channel) in inputs.iter().enumerate() {
            // iterate over samples
            for sam_i in 0..channel.len() {
                // write dsp function output to the output buffer
                outputs[chan_i][sam_i] = self.dsp_fn(inputs[chan_i][sam_i], chan_i == 0);
            }
        }
    }


    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.current_function as f32 / FUNCTIONS.iter().len() as f32,
            1 => self.parameter_a,
            2 => self.parameter_b,
            3 => self.parameter_c,
            4 => self.parameter_d,
            5 => self.feedback,
            6 => self.gain,
            7 => self.stereo_depth,
            8 => self.stereo_color,
            9 => self.beta,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.current_function = ((value - 0.001) * self.functions_len as f32) as usize,
            1 => self.parameter_a = value,
            2 => self.parameter_b = value,
            3 => self.parameter_c = value,
            4 => self.parameter_d = value,
            5 => self.feedback = value,
            6 => self.gain = value,
            7 => self.stereo_depth = value,
            8 => self.stereo_color = value,
            9 => self.beta = value,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "function".to_string(),
            1 => "parameter a".to_string(),
            2 => "parameter b".to_string(),
            3 => "parameter c".to_string(),
            4 => "parameter d".to_string(),
            5 => "feedback".to_string(),
            6 => "gain".to_string(),
            7 => "stereo".to_string(),
            8 => "stereo freq".to_string(),
            9 => "beta".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => FUNCTIONS[self.current_function].1.to_string(),
            1 => stringify!(self.parameter_a).to_string(),
            2 => stringify!(self.parameter_b).to_string(),
            3 => stringify!(self.parameter_c).to_string(),
            4 => stringify!(self.parameter_d).to_string(),
            5 => stringify!(self.feedback).to_string(),
            6 => stringify!(self.gain).to_string(),
            7 => stringify!(self.stereo_depth).to_string(),
            8 => stringify!(self.stereo_color).to_string(),
            _ => "".to_string(),
        }
    }

    fn new(_: HostCallback) -> Self {
        FeedbackWS {
            last_sample_l: 0.0,
            last_sample_r: 0.0,
            feedback: 0.5,

            parameter_a: 0.0,
            parameter_b: 0.0,
            parameter_c: 0.0,
            parameter_d: 0.0,
            current_function: 0,
            gain: 0.5,
            stereo_depth: 0.0,
            stereo_color: 0.1,
            beta: 0.99,

            functions_len: FUNCTIONS.iter().len(),
        }
    }
}

impl FeedbackWS {
    /// main dsp function, takes a single sample and returns a single sample
    /// the left argument is true if the sample is from the left chanel
    fn dsp_fn(&mut self, input: f32, left: bool) -> f32 {
        let mut pipe = input;
        // feedback is added to the audio pipe
        pipe += if left {
            self.last_sample_l
        } else {
            self.last_sample_r
        } * self.feedback;
        pipe *= self.gain;
        // waveshaping 
        pipe = self.waveshape(pipe);
        // stereoshaping
        pipe = self.stereoshape(pipe, left);
        // unfiltered signal is stored
        let unfiltered = pipe;
        // hpf with very small beta value as DC blocker
        pipe = self.hpf(pipe, left, 0.0001);
        // sample is stored as last_sample_x and gets highpass filtered before.
        if left {
            self.last_sample_l = self.hpf(unfiltered, left, self.beta);
        } else {
            self.last_sample_r = self.hpf(unfiltered, !left, self.beta);
        }
        return pipe;
    }

    /// waveshaping function
    fn waveshape(&self, input: f32) -> f32 {
        FUNCTIONS[self.current_function].0(input, self.parameter_a, self.parameter_b, self.parameter_c, self.parameter_d)
    }

    /// stereoshaping function, adds a sinoid shaping which is invertet on the right channel for
    /// stereo widht
    fn stereoshape(&self, input: f32, left: bool) -> f32 {
        input + (input * 1000.0 * self.stereo_color).sin() 
            * self.stereo_depth * if left { 1.0 } else { -1.0 }
    }

    /// simple highpass filter
    fn hpf(&self, input: f32, left: bool, beta: f32) -> f32 {
        let last = if left {
            self.last_sample_l
        } else {
            self.last_sample_r
        };
        input - (input * beta + last * (1.0 - beta))
    }
}

plugin_main!(FeedbackWS);
