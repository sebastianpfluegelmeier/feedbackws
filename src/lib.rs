#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin, HostCallback};
use vst2::buffer::AudioBuffer;

static FUNCTIONS: &'static [(fn(f32, f32) -> f32, &str)] = &[
    (analog_dist,     "2(1/1+e^(-a*x)))-1"),
    (sin_log,         "sin(a*log(x+1))"),
    (x_sin_x_squared, "x * sin(x^2 + a))"),
    (sin_fun,         "x * (a + (1 - a)) * (1 + sin(x * a * 200))"),
];

fn analog_dist(sig: f32, param: f32) -> f32 {
    2.0 * (1.0 / (1.0 + std::f32::consts::E.powf(-param * sig * 10.0))) - 1.0
}

fn sin_log(sig: f32, param: f32) -> f32 {
    (30.0 * param * (sig + 1.0).ln()).sin()
}

fn x_sin_x_squared(sig: f32, param: f32) -> f32 {
    sig * f32::sin((sig.powi(2) + param * 3.0))
}

fn sin_fun(sig: f32, param: f32) -> f32 {
    sig * (param + (1.0 - param) * (1.0 + f32::sin(sig * param * 200.0)))   
}

#[derive(Default)]
struct FeedbackWS {
    last_sample_l: f32,
    last_sample_r: f32,

    feedback: f32,
    parameter: f32,
    gain: f32,
    stereo: f32,
    stereo_freq: f32,
    beta: f32,
    current_function: usize,
    functions_len: usize,
}

impl Plugin for FeedbackWS {
    fn get_info(&self) -> Info {
        Info {
            name: "FeedbackWS".to_string(),
            unique_id: 543229834,
            inputs: 2,
            outputs: 2,
            parameters: 7,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();
        for (chan_i, channel) in inputs.iter().enumerate() {
            for sam_i in 0..channel.len() {
                outputs[chan_i][sam_i] = self.dsp_fn(inputs[chan_i][sam_i], chan_i == 0);
            }
        }
    }


    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.current_function as f32 / FUNCTIONS.iter().len() as f32,
            1 => self.parameter,
            2 => self.feedback,
            3 => self.gain,
            4 => self.stereo,
            5 => self.stereo_freq,
            6 => self.beta,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.current_function = ((value - 0.001) * self.functions_len as f32) as usize,
            1 => self.parameter = value,
            2 => self.feedback = value,
            3 => self.gain = value,
            4 => self.stereo = value,
            5 => self.stereo_freq = value,
            6 => self.beta = value,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "function".to_string(),
            1 => "function parameter".to_string(),
            2 => "feedback".to_string(),
            3 => "gain".to_string(),
            4 => "stereo".to_string(),
            5 => "stereo freq".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => FUNCTIONS[self.current_function].1.to_string(),
            1 => stringify!(self.parameter).to_string(),
            2 => stringify!(self.feedback).to_string(),
            3 => stringify!(self.gain).to_string(),
            4 => stringify!(self.stereo).to_string(),
            5 => stringify!(self.stereo_freq).to_string(),
            _ => "".to_string(),
        }
    }

    fn new(_: HostCallback) -> Self {
        FeedbackWS {
            last_sample_l: 0.0,
            last_sample_r: 0.0,
            feedback: 0.5,

            parameter: 0.0,
            current_function: 0,
            gain: 0.5,
            stereo: 0.0,
            stereo_freq: 0.1,
            beta: 0.99,

            functions_len: FUNCTIONS.iter().len(),
        }
    }
}

impl FeedbackWS {
    fn dsp_fn(&mut self, input: f32, left: bool) -> f32 {
        let mut pipe = input;
        pipe += if left {
            self.last_sample_l
        } else {
            self.last_sample_r
        } * self.feedback;
        pipe *= self.gain;
        pipe = self.waveshape(pipe);
        pipe = self.stereoshape(pipe, left);
        let unfiltered = pipe;
        pipe = self.hpf(pipe, left, 0.0001);
        if left {
            self.last_sample_l = self.hpf(unfiltered, left, self.beta);
        } else {
            self.last_sample_r = self.hpf(unfiltered, !left, self.beta);
        }
        return pipe;
    }

    fn waveshape(&self, input: f32) -> f32 {
        FUNCTIONS[self.current_function].0(input, self.parameter)
    }

    fn stereoshape(&self, input: f32, left: bool) -> f32 {
        input + (input * 1000.0 * self.stereo_freq).sin() 
            * self.stereo * if left { 1.0 } else { -1.0 }
    }

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
