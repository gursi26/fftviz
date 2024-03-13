use crate::{cli_args_to_fft_args, config_path, parse_cli_args, reset_config_file, write_fftargs_to_config, AppState, FFTArgs};
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


pub fn ui_example_system(
    mut contexts: EguiContexts,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<AppState>,
    mut args: ResMut<FFTArgs>,
) {
    if args.display_gui {
        let window_handle = egui::Window::new("")
            .fixed_size(egui::Vec2 { x: 100.0, y: 100.0 })
            .anchor(Align2::RIGHT_TOP, egui::Vec2::new(-10.0, 10.0))
            .collapsible(false);

        window_handle.show(contexts.ctx_mut(), |ui| {
            ui.checkbox(&mut args.track_name, "Display title: ");
            if args.track_name {
                ui.horizontal(|ui| {
                    ui.label("Text color: ");
                    color_picker_widget(ui, &mut args.text_color);
                });
                ui.horizontal(|ui| {
                    ui.label("Text size: ");
                    ui.add(egui::Slider::new(&mut args.font_size, 10..=50).text("value"));
                });
            }

            ui.horizontal(|ui| {
                ui.label("Bar color: ");
                color_picker_widget(ui, &mut args.bar_color);
            });

            ui.horizontal(|ui| {
                ui.label("Border color: ");
                color_picker_widget(ui, &mut args.border_color);
            });

            ui.horizontal(|ui| {
                ui.label("Background color: ");
                color_picker_widget(ui, &mut args.background_color);
            });

            ui.horizontal(|ui| {
                ui.label("Border size: ");
                ui.add(egui::Slider::new(&mut args.border_size, 0..=10).text("value"));
            });

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    write_fftargs_to_config(&args);
                    app_state.display_str = format!("Saved to {:?}", config_path());
                    app_state.stopwatch.start();
                }
                if ui.button("Reset").clicked() {
                    *args = parse_cli_args();
                    app_state.display_str = String::from("Reset to saved settings.");
                    app_state.stopwatch.start();
                    args.display_gui = true;
                }
                if ui.button("Reset to default").clicked() {
                    *args = cli_args_to_fft_args(crate::args::CLIArgs::parse(), true);
                    app_state.display_str = String::from("Reset to default settings.");
                    app_state.stopwatch.start();
                    args.display_gui = true;
                }
            });

            if app_state.stopwatch.is_running() {
                ui.label(&app_state.display_str);
            }
            if app_state.stopwatch.elapsed().as_secs() > 3 {
                app_state.stopwatch.stop();
                app_state.stopwatch.reset();
            }
        });
    }
}

fn color_picker_widget(ui: &mut egui::Ui, color: &mut Color) -> egui::Response {
    let [r, g, b, a] = color.as_rgba_f32();
    let mut egui_color: egui::Rgba = egui::Rgba::from_srgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );
    let res = egui::widgets::color_picker::color_edit_button_rgba(
        ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );
    let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
    *color = Color::rgba(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    );
    res
}

