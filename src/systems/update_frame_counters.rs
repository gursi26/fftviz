use crate::*;
use bevy::prelude::*;

pub fn update_frame_counters(mut fft_state: ResMut<FFTState>) {
    let elapsed_time = fft_state.fft_timer.elapsed().as_secs_f64();
    fft_state.fft_frame_counter = (elapsed_time / TIME_BETWEEN_FFT_FRAMES) as usize;
    fft_state.total_frame_counter = (elapsed_time / TIME_BETWEEN_FRAMES) as usize;
}
