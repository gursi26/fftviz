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
    mut differencing_args_query: Query<&mut FFTArgs>,
) {
    let mut differencing_args = differencing_args_query.get_single_mut().unwrap();

    // Update text color + visibility + size
    if differencing_args.text_color != args.text_color || differencing_args.track_name != args.track_name || differencing_args.font_size != args.font_size {
        for mut text in &mut text_query {
            if args.track_name {
                text.sections[0].style.color = args.text_color;
                text.sections[0].style.font_size = args.font_size as f32;
            } else {
                text.sections[0].style.color = Color::rgba(0.0, 0.0, 0.0, 0.0);
            }
        }

        differencing_args.text_color = args.text_color;
        differencing_args.font_size = args.font_size;
        differencing_args.track_name = args.track_name;
    }

    // Update background color
    if differencing_args.background_color != args.background_color {
        clear_color.0 = args.background_color;
        differencing_args.background_color = args.background_color;
    }

    // Update bar colors
    if differencing_args.bar_color != args.bar_color || differencing_args.border_color != args.border_color {
        for handle in app_state.curr_bars.chunks(2) {
            let (color_handle1, color_handle2) = (handle[0].1.clone_weak(), handle[1].1.clone_weak());
            materials.get_mut(color_handle1).unwrap().color = args.border_color;
            materials.get_mut(color_handle2).unwrap().color = args.bar_color;
        }
        differencing_args.bar_color = args.bar_color;
        differencing_args.border_color = args.border_color;
    }

    // Update border size
    if differencing_args.border_size != args.border_size {
        let w = window.single_mut().width();
        let bar_size = ((w as f32 / (app_state.curr_bars.len() / 2) as f32) - args.border_size as f32) / 2.0;

        for handle in app_state.curr_bars.chunks(2) {
            let handle1 = handle[1].0.clone_weak();

            let dims = meshes
                .get_mut(handle1)
                .unwrap()
                .attribute_mut(Mesh::ATTRIBUTE_POSITION)
                .unwrap();

            match dims {
                VertexAttributeValues::Float32x3(x) => {
                    x[0][0] = bar_size;
                    x[1][0] = -bar_size;
                    x[2][0] = -bar_size;
                    x[3][0] = bar_size;
                }
                _ => {}
            }
        }

        differencing_args.border_size = args.border_size;
    }
}
