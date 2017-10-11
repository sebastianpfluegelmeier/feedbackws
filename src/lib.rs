#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin};
use vst2::buffer::AudioBuffer;

#[derive(Default)]
struct FeedbackWS {
    last_sample_l: f32,
    last_sample_r: f32,
    feedback: f32,
    waveshape_points: usize,
    waveshape_table: Vec<f32>,
}

impl Plugin for FeedbackWs {
    fn get_info(&self) -> Info {
        Info {
            name: "FeedbackWs".to_string(),
            unique_id: 5432,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: AudioBuffer<f32>) {
        let (inputs, mut outputs) = buffer.split();
        for (chan_i, channel) in inputs.iter().enumerate() {
            for (sam_i, sample) in channel.iter().enumerate() {
                outputs[chan_i][sam_i] = self.dsp_fn(inputs[chan_i][sam_i], chan_i == 0);
            }
        }
    }

}

impl FeedbackWs {
    fn dsp_fn(&mut self, input: f32, left: bool) -> f32 {
        let neg_input = input < 0.0;
        let mut pipe = 0.0;
        pipe += if left {self.last_sample_l} else {self.last_sample_r} * self.feedback;
        pipe = if neg_input {-pipe} else {pipe};
        pipe = self.waveshape(pipe, left);
        pipe = if neg_input {-pipe} else {pipe};
        if left {
            self.last_sample_l = pipe;
        } else {
            self.last_sample_r = pipe;
        }
        return pipe;
    }

    fn waveshape(&self, input: f32, left: bool) -> f32 {
        let float_index: f32 = (input * self.waveshape_points as f32);
        let floor_index: f32 = float_index.floor();
        let ceil_index:  f32 = float_index.ceil();
        let ceil_frac:   f32 = floor_index - float_index;
        let floor_frac:  f32 = 1.0 - floor_index;
        self.waveshape_table[floor_index as usize] * floor_frac + 
        self.waveshape_table[ceil_index  as usize] * ceil_frac

    }
}

plugin_main!(FeedbackWs);
