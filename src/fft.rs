use crate::*;
use bincode::{deserialize, serialize};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rodio::{source::Source, Decoder, OutputStream};
use serde::{Deserialize, Serialize};
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hamming_window;
use spectrum_analyzer::{samples_fft_to_spectrum, FrequencyLimit};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct FFT {
    pub fft: Vec<Vec<f32>>,
    pub num_frames: usize,
    pub num_bars: usize,
    pub min: f32,
    pub max: f32,
}

pub fn time_interpolate(v1: &Vec<f32>, v2: &Vec<f32>, alpha: f32) -> Vec<f32> {
    v1.iter()
        .zip(v2.iter())
        .map(|(x, y)| x.clone() * (1.0 - alpha) + y.clone() * alpha)
        .collect::<Vec<f32>>()
}

pub fn space_interpolate(v: &mut Vec<f32>, num_new_frames: u32) {
    let l = v.len();
    for i in (0..(l - 1)).rev() {
        let curr = v[i];
        let next = v[i + 1];
        let diff = next - curr;
        for j in (0..num_new_frames).rev() {
            v.insert(
                i + 1,
                curr + diff * ((j as f32 + 1.0) / (num_new_frames + 1) as f32),
            );
        }
    }
}

#[allow(dead_code)]
pub fn smooth_fft(mut fft: FFT, alpha: u32) -> FFT {
    let mut new_fft = Vec::new();
    for i in (alpha as usize)..(fft.num_frames - alpha as usize) {
        let mut new_frame = fft.fft[i].clone();
        new_frame.iter_mut().enumerate().for_each(|(j, x)| {
            *x = ((i - alpha as usize)..(i + alpha as usize))
                .into_iter()
                .map(|i| fft.fft[i][j])
                .sum::<f32>() as f32 / (2.0 * alpha as f32 + 1.0)
        });
        new_fft.push(new_frame);
    }
    fft.fft = new_fft;
    fft
}

pub fn normalize_fft(mut fft: FFT, bounds: &[f32], scaling_factor: &[f32]) -> FFT {
    let min_max_scale = fft.max - fft.min;
    let rescale = |mut x: Vec<f32>| -> Vec<f32> {
        for i in x.iter_mut() {
            *i = (*i - fft.min) / min_max_scale;
            for (bound, scale) in bounds.iter().zip(scaling_factor.iter()) {
                if *i < *bound {
                    *i *= scale;
                    break;
                }
            }
        }
        x
    };
    fft.fft = fft.fft.into_par_iter().map(|x| rescale(x)).collect();
    fft
}

#[allow(dead_code)]
pub fn write_fft_to_binary_file(filepath: &PathBuf, fft: &FFT) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    let encoded_data = serialize(fft).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    file.write_all(&encoded_data)?;
    Ok(())
}

#[allow(dead_code)]
pub fn read_fft_from_binary_file(filepath: &PathBuf) -> io::Result<FFT> {
    let mut file = File::open(filepath)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let fft: FFT = deserialize(&buffer).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(fft)
}

pub fn compute_fft(audio_path: &PathBuf) -> FFT {
    let (_stream, _) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(audio_path).unwrap());
    let source = Decoder::new(file).unwrap();

    let n_channels = source.channels() as i32;
    let sample_rate = source.sample_rate() as i32;

    let mut source = source.peekable();

    let step_amount = ((sample_rate * n_channels) as usize / FFT_FPS as usize) as i32 - FFT_WINDOW;
    let (mut min, mut max): (f32, f32) = (100.0, 0.0);
    let mut output_vec = Vec::new();

    while source.peek().is_some() {
        let mut frame = Vec::new();
        for _ in 0..FFT_WINDOW {
            frame.push(source.next().unwrap())
        }

        for _ in 0..step_amount {
            source.next();
        }

        let mut samples = [0.0; FFT_WINDOW as usize];
        for (i, stereo) in frame.chunks(n_channels as usize).enumerate() {
            samples[i] = stereo
                .iter()
                .map(|x| x.clone() as f32 * 20.0 / n_channels as f32)
                .sum::<f32>();
        }

        let hann_window = hamming_window(&samples);

        let spectrum_hann_window = samples_fft_to_spectrum(
            &hann_window,
            sample_rate as u32,
            FrequencyLimit::Range(FREQ_WINDOW_LOW, FREQ_WINDOW_HIGH),
            Some(&divide_by_N_sqrt),
        )
        .unwrap();

        let curr_vec = spectrum_hann_window
            .data()
            .into_iter()
            .map(|(_, fval)| fval.val())
            .collect::<Vec<f32>>();

        for val in curr_vec.iter() {
            max = max.max(*val);
            min = min.min(*val);
        }
        output_vec.push(curr_vec.clone());
    }

    let num_frames = output_vec.len();
    let num_bars = output_vec[0].len();
    FFT {
        fft: output_vec,
        num_frames,
        num_bars,
        min,
        max,
    }
}
