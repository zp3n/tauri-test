use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use std::sync::{Arc, Mutex};

pub struct AudioEngine {
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            input_stream: None,
            output_stream: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        let host = cpal::default_host();

        let input_device = host
            .default_input_device()
            .ok_or("入力デバイスが見つかりません")?;

        let output_device = host
            .default_output_device()
            .ok_or("出力デバイスが見つかりません")?;

        let input_config = input_device
            .default_input_config()
            .map_err(|e| e.to_string())?;

        let output_config = output_device
            .default_output_config()
            .map_err(|e| e.to_string())?;

        println!(
            "Input: {}",
            input_device
                .description()
                .map(|d| d.to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        );

        println!(
            "Output: {}",
            output_device
                .description()
                .map(|d| d.to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        );

        println!(
            "Input: {} Hz / {:?}",
            input_config.sample_rate(),
            input_config.sample_format()
        );

        println!(
            "Output: {} Hz / {:?}",
            output_config.sample_rate(),
            output_config.sample_format()
        );

        /*
         * 今回はまず、
         *
         * Input → buffer → Output
         *
         * の最小構成を作る。
         */

        let buffer: Arc<Mutex<Vec<f32>>> =
            Arc::new(Mutex::new(Vec::with_capacity(4096)));

        let input_buffer = Arc::clone(&buffer);
        let output_buffer = Arc::clone(&buffer);

        let input_stream = match input_config.sample_format() {
            SampleFormat::F32 => {
                let config: StreamConfig = input_config.clone().into();

                input_device
                    .build_input_stream(
                        config,
                        move |data: &[f32], _| {
                            if let Ok(mut buffer) = input_buffer.try_lock() {
                                buffer.extend_from_slice(data);

                                // 暴走防止
                                if buffer.len() > 16384 {
                                    let excess = buffer.len() - 8192;
                                    buffer.drain(0..excess);
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Input stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            SampleFormat::I16 => {
                let config: StreamConfig = input_config.clone().into();

                input_device
                    .build_input_stream(
                        config,
                        move |data: &[i16], _| {
                            if let Ok(mut buffer) = input_buffer.try_lock() {
                                for &sample in data {
                                    buffer.push(sample as f32 / i16::MAX as f32);
                                }

                                if buffer.len() > 16384 {
                                    let excess = buffer.len() - 8192;
                                    buffer.drain(0..excess);
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Input stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            SampleFormat::U16 => {
                let config: StreamConfig = input_config.clone().into();

                input_device
                    .build_input_stream(
                        config,
                        move |data: &[u16], _| {
                            if let Ok(mut buffer) = input_buffer.try_lock() {
                                for &sample in data {
                                    let sample =
                                        sample as f32 / u16::MAX as f32;

                                    buffer.push(sample * 2.0 - 1.0);
                                }

                                if buffer.len() > 16384 {
                                    let excess = buffer.len() - 8192;
                                    buffer.drain(0..excess);
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Input stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            _ => {
                return Err("未対応の入力フォーマットです".into());
            }
        };

        let output_stream = match output_config.sample_format() {
            SampleFormat::F32 => {
                let config: StreamConfig = output_config.clone().into();

                output_device
                    .build_output_stream(
                        config,
                        move |data: &mut [f32], _| {
                            if let Ok(mut buffer) = output_buffer.try_lock() {
                                for sample in data.iter_mut() {
                                    *sample = buffer.pop().unwrap_or(0.0);
                                }
                            } else {
                                for sample in data.iter_mut() {
                                    *sample = 0.0;
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Output stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            SampleFormat::I16 => {
                let config: StreamConfig = output_config.clone().into();

                output_device
                    .build_output_stream(
                        config,
                        move |data: &mut [i16], _| {
                            if let Ok(mut buffer) = output_buffer.try_lock() {
                                for sample in data.iter_mut() {
                                    let value =
                                        buffer.pop().unwrap_or(0.0);

                                    *sample =
                                        (value * i16::MAX as f32) as i16;
                                }
                            } else {
                                for sample in data.iter_mut() {
                                    *sample = 0;
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Output stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            SampleFormat::U16 => {
                let config: StreamConfig = output_config.clone().into();

                output_device
                    .build_output_stream(
                        config,
                        move |data: &mut [u16], _| {
                            if let Ok(mut buffer) = output_buffer.try_lock() {
                                for sample in data.iter_mut() {
                                    let value =
                                        buffer.pop().unwrap_or(0.0);

                                    let value =
                                        ((value + 1.0) * 0.5)
                                            * u16::MAX as f32;

                                    *sample = value as u16;
                                }
                            } else {
                                for sample in data.iter_mut() {
                                    *sample = u16::MAX / 2;
                                }
                            }
                        },
                        move |err| {
                            eprintln!("Output stream error: {err}");
                        },
                        None,
                    )
                    .map_err(|e| e.to_string())?
            }

            _ => {
                return Err("未対応の出力フォーマットです".into());
            }
        };

        input_stream
            .play()
            .map_err(|e| e.to_string())?;

        output_stream
            .play()
            .map_err(|e| e.to_string())?;

        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);

        Ok(())
    }

    pub fn stop(&mut self) {
        self.input_stream = None;
        self.output_stream = None;
    }
}