use crate::*;
use crate::fft::time_interpolate;
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::egui::{Align2, Color32, Stroke};
use bevy::sprite::{Anchor, Material2d};
use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use clap::{ArgAction, Parser};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rodio::{source::Source, Decoder, OutputStream};
use std::ffi::OsString;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;


pub fn update_view_settings (
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut app_state: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
    mut clear_color: ResMut<ClearColor>,
    mut text_query: Query<&mut Text>,
) {
    // Update text color + visibility + size
    for mut text in &mut text_query {
        if args.track_name {
            text.sections[0].style.color = args.text_color;
            text.sections[0].style.font_size = args.font_size as f32;
        } else {
            text.sections[0].style.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
        }
    }

    // Update background color
    clear_color.0 = args.background_color;

    // Update bar colors
    for handle in app_state.curr_bars.chunks(2) {
        let (color_handle1, color_handle2) = (handle[0].1.clone_weak(), handle[1].1.clone_weak());
        materials.get_mut(color_handle1).unwrap().color = args.border_color;
        materials.get_mut(color_handle2).unwrap().color = args.bar_color;
    }
}
