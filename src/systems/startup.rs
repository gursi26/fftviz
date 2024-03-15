use crate::{FFTArgs, FFTState};
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

fn spawn_bars(
    num_bars: u32,
    w: f32,
    args: &FFTArgs,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> (Vec<(Handle<Mesh>, Handle<ColorMaterial>)>, Vec<Entity>) {
    let bar_size = w as f32 / num_bars as f32;
    let mut handle_vec = Vec::new();
    let mut despawn_handles = Vec::new();

    for i in 0..num_bars {
        let handle1 = meshes.add(Rectangle::new(bar_size, 0.0));
        let color_handle = materials.add(args.border_color);
        handle_vec.push((handle1.clone(), color_handle.clone()));

        let dh = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(handle1),
                material: color_handle,
                transform: Transform::from_xyz(
                    bar_size * i as f32 + (bar_size / 2.0) - (w / 2.0) as f32,
                    0.0,
                    -1.0,
                ),
                ..default()
            })
            .id();
        despawn_handles.push(dh);

        let handle2 = meshes.add(Rectangle::new(bar_size - args.border_size as f32, 0.0));
        let color_handle = materials.add(args.bar_color);
        handle_vec.push((handle2.clone(), color_handle.clone()));

        let dh = commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(handle2),
                material: color_handle,
                transform: Transform::from_xyz(
                    bar_size * i as f32 + (bar_size / 2.0) - (w / 2.0) as f32,
                    0.0,
                    0.0,
                ),
                ..default()
            })
            .id();
        despawn_handles.push(dh);
    }
    (handle_vec, despawn_handles)
}

pub fn startup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<FFTState>,
    clear_color: Res<ClearColor>,
    args: Res<FFTArgs>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut differencing_args = args.clone();
    differencing_args.track_name = !args.track_name;

    commands.spawn(differencing_args);

    let w = window.single_mut().width();
    let h = window.single_mut().height();

    let text_style = TextStyle {
        font: Default::default(),
        font_size: args.font_size as f32,
        color: args.text_color,
    };

    commands.spawn(Text2dBundle {
        text: Text::from_section(
            format!(
                "Playing: \"{}\"",
                args.file_path.file_name().unwrap().to_str().unwrap()
            ),
            text_style.clone(),
        ),
        transform: Transform::from_xyz(-(w as f32 / 2.0) + 10.0, (h as f32 / 2.0) - 10.0, 0.0),
        text_anchor: Anchor::TopLeft,
        ..default()
    });

    let num_bars = fft_queue.fft[0].len();

    let (mesh_handles, despawn_handles) = spawn_bars(
        num_bars as u32,
        w,
        &args,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    fft_queue.curr_bars = mesh_handles;
    fft_queue.despawn_handles = despawn_handles;
}
