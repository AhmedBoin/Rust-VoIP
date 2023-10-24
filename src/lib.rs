use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;
use std::sync::{Arc, Mutex};

pub struct VoIP {
    microphone: Stream,
    stereo: Stream,
    microphone_buffer: Arc<Mutex<Vec<f32>>>,
    stereo_buffer: Arc<Mutex<Vec<f32>>>,
}

fn error_callback(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn convert_f32_to_u8(input: Vec<f32>) -> Vec<u8> {
    let mut output: Vec<u8> = vec![];

    for &value in input.iter() {
        let bytes = value.to_le_bytes();
        output.extend_from_slice(&bytes);
    }

    output
}

fn convert_u8_to_f32(input: &[u8]) -> Vec<f32> {
    let mut output: Vec<f32> = vec![];

    for chunk in input.chunks(4) {
        if chunk.len() == 4 {
            let mut bytes = [0; 4];
            bytes.copy_from_slice(chunk);
            let value = f32::from_le_bytes(bytes);
            output.push(value);
        }
    }

    output
}

#[rustfmt::skip]
impl VoIP {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let input_device = host.default_input_device().unwrap();
        let output_device = host.default_output_device().unwrap();

        let config: cpal::StreamConfig = input_device.default_input_config().unwrap().into();

        let microphone_buffer = Arc::new(Mutex::new(vec![]));
        let rx = microphone_buffer.clone();
        let stereo_buffer = Arc::new(Mutex::new(vec![]));
        let tx = stereo_buffer.clone();

        let data_callback = move |data: &[f32], _: &_| *rx.lock().unwrap() = data.to_vec();
        let microphone = input_device.build_input_stream(&config, data_callback, error_callback, None).unwrap();

        let data_callback = move |data: &mut [f32], _: &_| {
            let mut ab = tx.lock().unwrap();
            if !ab.is_empty() {
                let bytes_to_write = std::cmp::min(data.len(), ab.len());
                data[..bytes_to_write].copy_from_slice(&ab[..bytes_to_write]);
                ab.drain(0..bytes_to_write);
            }
        };
        let stereo = output_device.build_output_stream(&config, data_callback, error_callback, None).unwrap();

        Self {
            microphone,
            stereo,
            microphone_buffer,
            stereo_buffer,
        }
    }

    pub fn start_microphone(&self) {
        self.microphone.play().unwrap();
    }

    pub fn stop_microphone(&self) {
        self.microphone.pause().unwrap();
    }

    pub fn start_stereo(&self) {
        self.stereo.play().unwrap();
    }

    pub fn stop_stereo(&self) {
        self.stereo.pause().unwrap();
    }

    pub fn recv(&self, data: &[u8]) {
        *self.stereo_buffer.lock().unwrap() = convert_u8_to_f32(data);
    }

    pub fn send(&self) -> Vec<u8> {
        convert_f32_to_u8(self.microphone_buffer.lock().unwrap().clone())
    }
}
