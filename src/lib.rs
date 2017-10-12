#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin, HostCallback};
use vst2::buffer::AudioBuffer;

static FUNCTIONS: &'static [fn (f32, f32) -> f32] =  &[analog_dist, sin_log];

fn analog_dist(sig: f32, param:f32) -> f32 {
        2.0 * (1.0/(1.0 + std::f32::consts::E.powf(-param * sig))) - 1.0
}

fn sin_log(sig: f32, param:f32) -> f32 {
    (param*(sig + 1.0).log(std::f32::consts::E)).sin()
}

#[derive(Default)]
struct FeedbackWS {
    last_sample_l: f32,
    last_sample_r: f32,

    feedback: f32,
    parameter: f32,
    gain: f32,
    current_function: usize,
    functions_len: usize,
}

impl Plugin for FeedbackWS {
    fn get_info(&self) -> Info {
        Info {
            name: "FeedbackWS".to_string(),
            unique_id: 5432,
            inputs: 2,
            outputs: 2,
            parameters: 3,
            ..Default::default()
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.current_function as f32 / FUNCTIONS.iter().len() as f32 ,
            1 => self.parameter,
            2 => self.feedback,
            3 => self.gain,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.current_function = ((value - 0.001) * self.functions_len as f32) as usize,
            1 => self.parameter = value,
            2 => self.feedback = value,
            3 => self.gain = value,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "function".to_string(),
            1 => "function parameter".to_string(),
            2 => "feedback".to_string(),
            3 => "gain".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => match self.current_function {
                0 => "analog",
                1 => "sinelog",
                _ => "unknown"
            }.to_owned(),
            1 => stringify!(self.parameter).to_string(),
            2 => stringify!(self.feedback).to_string(),
            3 => stringify!(self.gain).to_string(),
            _ => "".to_string(),
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

    fn new(_: HostCallback) -> Self {
        FeedbackWS {
            last_sample_l: 0.0,
            last_sample_r: 0.0,
            feedback: 0.5,

            parameter: 0.0,
            current_function: 0,
            gain: 0.5,

            functions_len: FUNCTIONS.iter().len(),
        }
    }
}

impl FeedbackWS {
    fn dsp_fn(&mut self, input: f32, left: bool) -> f32 {
        let mut pipe = input;
        pipe += if left {self.last_sample_l} else {self.last_sample_r} * self.feedback;
        pipe *= self.gain;
        pipe = self.waveshape(pipe, left);
        if left {
            self.last_sample_l = pipe;
        } else {
            self.last_sample_r = pipe;
        }
        return pipe;
    }

    fn waveshape(&self, input: f32, left: bool) -> f32 {
        FUNCTIONS[self.current_function](input, self.parameter)
    }

}

plugin_main!(FeedbackWS);
