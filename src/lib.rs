#[macro_use]
extern crate vst2;

use vst2::plugin::{Info, Plugin, HostCallback, CanDo};
use vst2::buffer::AudioBuffer;
use vst2::event::Event;
use vst2::api::Supported;

/// Array of (waveshape function, descriptor string) tuples.
static FUNCTIONS: &'static [(fn(f32, f32, f32) -> f32, &str)] = &[
    (analog_dist,     "2(1/1+e^(-a*x)))-1"),
    (sin_log,         "sin(a*log(x+1))"),
    (x_sin_x_squared, "x * sin(x^2 + a))"),
    (sin_fun,         "x * (a + (1 - a)) * (1 + sin(x * b * 200))"),
    (fm_logis,        "2 * (1 / 1 + e ^ (-x * a * cos (b * 50 / (2 * pi)))) - 1"),
];

// All waveshape functions
fn analog_dist(sig: f32, a: f32, _: f32) -> f32 {
    2.0 * (1.0 / (1.0 + std::f32::consts::E.powf(-a * sig * 10.0))) - 1.0
}

fn sin_log(sig: f32, a: f32, _: f32) -> f32 {
    (30.0 * a * (sig + 1.0).ln()).sin()
}

fn x_sin_x_squared(sig: f32, a: f32, _: f32) -> f32 {
    sig * f32::sin((sig.powi(2) + a * 3.0))
}

fn sin_fun(sig: f32, a: f32, b: f32) -> f32 {
    sig * (a + (1.0 - a) * (1.0 + f32::sin(sig * b * 200.0)))   
}

fn fm_logis(sig: f32, a: f32, b: f32) -> f32 {
    2.0 * (1.0 / (1.0 + std::f32::consts::E.powf(-sig * a * f32::cos(b * 50.0/(2.0 * std::f32::consts::PI))))) - 1.0
}

fn midi_note_to_hz(note: u8) -> f64 {
    (440.0/32.0) * (note as f64)
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
    gain: f32,
    stereo_depth: f32,
    stereo_color: f32,
    beta: f32,
    current_function: usize,
    // length of FUNCTIONS array
    functions_len: usize,
    sample_rate: f64,
    notes: Vec<Option<u8>>,
}

impl Plugin for FeedbackWS {
    fn get_info(&self) -> Info {
        Info {
            name: "FeedbackWS".to_string(),
            unique_id: 543229834,
            inputs: 2,
            outputs: 2,
            parameters: 8,
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

    fn process_events(&mut self, events: Vec<Event>) {
        for event in events {
            match event {
                Event::Midi { data, .. } => self.process_event(data),
                _ => {}
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate as f64;
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe
        }
    }

    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.current_function as f32 / FUNCTIONS.iter().len() as f32,
            1 => self.parameter_a,
            2 => self.parameter_b,
            3 => self.feedback,
            4 => self.gain,
            5 => self.stereo_depth,
            6 => self.stereo_color,
            7 => self.beta,
            _ => 0.0,
        }
    }

    fn set_parameter(&mut self, index: i32, value: f32) {
        match index {
            0 => self.current_function = ((value - 0.001) * self.functions_len as f32) as usize,
            1 => self.parameter_a = value,
            2 => self.parameter_b = value,
            3 => self.feedback = value,
            4 => self.gain = value,
            5 => self.stereo_depth = value,
            6 => self.stereo_color = value,
            7 => self.beta = value,
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "function".to_string(),
            1 => "parameter a".to_string(),
            2 => "parameter b".to_string(),
            3 => "feedback".to_string(),
            4 => "gain".to_string(),
            5 => "stereo".to_string(),
            6 => "stereo freq".to_string(),
            7 => "beta".to_string(),
            _ => "".to_string(),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => FUNCTIONS[self.current_function].1.to_string(),
            1 => stringify!(self.parameter_a).to_string(),
            2 => stringify!(self.parameter_b).to_string(),
            3 => stringify!(self.feedback).to_string(),
            4 => stringify!(self.gain).to_string(),
            5 => stringify!(self.stereo_depth).to_string(),
            6 => stringify!(self.stereo_color).to_string(),
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
            current_function: 0,
            gain: 0.5,
            stereo_depth: 0.0,
            stereo_color: 0.1,
            beta: 0.99,

            sample_rate: 44100.0,
            functions_len: FUNCTIONS.iter().len(),
            notes: Vec::new(),
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
        FUNCTIONS[self.current_function].0(input, self.parameter_a, self.parameter_b)
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

    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    fn process_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_on(data[1]),
            144 => self.note_off(data[1]),
            _ => ()
        }
    }

    fn note_on(&mut self, note: u8) {
        for i in 0..self.notes.len() {
            if let None = self.notes[i] {
                self.notes[i] = Some(note);
            }
        }
    }

    fn note_off(&mut self, note: u8) {
        for i in 0..self.notes.len() {
            if let Some(x) = self.notes[i] {
                if x == note {
                    self.notes[i] = None;
                }
            }
        }
    }
}

plugin_main!(FeedbackWS);
