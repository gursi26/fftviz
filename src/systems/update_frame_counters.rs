use crate::fft::time_interpolate;
use crate::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::Anchor;
use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::egui::{Align2, Color32, Stroke};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::{ArgAction, Parser};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rodio::{source::Source, Decoder, OutputStream};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;


pub fn update_frame_counters(
    mut app_state: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
) {
    let elapsed_time = app_state.song_stopwatch.elapsed().as_secs_f64();
    app_state.fft_frame_counter = (elapsed_time / TIME_BETWEEN_FFT_FRAMES) as usize;
    app_state.total_frame_counter = (elapsed_time / TIME_BETWEEN_FRAMES) as usize;
}
