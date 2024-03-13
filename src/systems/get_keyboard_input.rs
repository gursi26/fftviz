use crate::{AppState, FFTArgs};
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::egui::{Align2, Color32, Stroke};
use bevy::sprite::Anchor;
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


pub fn get_keyboard_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut app_state: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.send(AppExit);
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        args.display_gui = !args.display_gui;
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        args.paused = !args.paused;
        if app_state.sink.is_paused() {
            app_state.sink.play();
        } else {
            app_state.sink.pause();
        }
    }
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        args.volume += 5;
        args.volume = args.volume.min(100);
        app_state.sink.set_volume(args.volume as f32 / 100.0);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        args.volume -= 5;
        args.volume = args.volume.max(0);
        app_state.sink.set_volume(args.volume as f32 / 100.0);
    }
}
