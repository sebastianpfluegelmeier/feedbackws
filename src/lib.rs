#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin};
use vst2::buffer::AudioBuffer;

#[derive(Default)]
struct VsTest {
    last_sample_l: f32,
    last_sample_r: f32,
    feedback: f32,
}

impl Plugin for VsTest {
    fn get_info(&self) -> Info {
        Info {
            name: "VsTest".to_string(),
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

impl VsTest {
    fn dsp_fn(&mut self, input: f32, left: bool) -> f32 {
        let neg_input = input < 0.0;
        let mut pipe = 0.0;
        pipe += if left {self.last_sample_l} else {self.last_sample_r} * self.feedback;
        pipe = self.waveshape(pipe, left);
        if left {
            self.last_sample_l = pipe;
        } else {
            self.last_sample_r = pipe;
        }
        return pipe;
    }

    fn waveshape(&self, input: f32, left: bool) -> f32 {
        0.0
    }
}

plugin_main!(VsTest);
