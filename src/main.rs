mod fft;
mod utils;

use fft::*;
use utils::*;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use clap::{ArgAction, Parser};
use rodio::{source::Source, Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use std::ffi::OsString;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use bevy::render::mesh::VertexAttributeValues;
use bevy::sprite::Anchor;


// Constants
const RENDERING_FPS: u32 = 60;
const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 700;

const FREQUENCY_RESOLUTION: u32 = 100;
const FFT_FPS: u32 = 12;
const FREQ_WINDOW_LOW: f32 = 0.0;
const FREQ_WINDOW_HIGH: f32 = 5000.0;
const FFT_WINDOW: i32 =
    ((256 as u64 / 107 as u64) * FREQUENCY_RESOLUTION as u64).next_power_of_two() as i32;
const BAR_INTERPOLATION_FACTOR: u32 = 1;
const RESCALING_THRESHOLDS: &[f32] = &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];
const RESCALING_FACTOR: &[f32] = &[2.0, 1.7, 1.3, 1.2, 1.1, 1.0, 0.9, 0.8, 0.7];


#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct CLIArgs {
    /// File path to Audio file
    #[arg()]
    file_path: String,

    /// Border size for each bar
    #[arg(long = "border-size", default_value_t = 1)]
    border_size: u32,

    /// Border color for each bar (in hex)
    #[arg(long = "border-color", default_value_t = String::from("000000"))]
    border_color: String,

    /// Color for each bar (in hex)
    #[arg(long = "bar-color", default_value_t = String::from("FF0000"))]
    bar_color: String,

    /// Whether to disable printing
    #[arg(long = "disable-title", action = ArgAction::SetTrue)]
    disable_title: bool,

    /// Color for currently playing text (in hex)
    #[arg(long = "text-color", default_value_t = String::from("FFFFFF"))]
    text_color: String,

    /// Font size of currently playing label
    #[arg(long = "font-size", default_value_t = 25)]
    font_size: u32,

    // Background color (in hex)
    #[arg(long = "background-color", default_value_t = String::from("000000"))]
    background_color: String,
}

#[derive(Resource)]
struct FFTArgs {
    file_path: PathBuf,
    border_size: i32,
    border_color: String,
    bar_color: String,
    disable_title: bool,
    text_color: String,
    font_size: i32,
    background_color: String,
}

#[derive(Resource)]
struct FFTQueue {
    fft: Vec<Vec<f32>>,
    curr_bars: Vec<Handle<Mesh>>,
    c: usize,
    i: usize,
}

fn update_bars(
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut fft_queue: ResMut<FFTQueue>,
    args: Res<FFTArgs>,
) {
    let h = window.single_mut().physical_height();
    let mut update_i = false;
    let interval = RENDERING_FPS / FFT_FPS;

    let curr_fft = match fft_queue.c as u32 % interval {
        0 => {
            if fft_queue.i > fft_queue.fft.len() {
                std::process::exit(0);
            }
            update_i = true;
            fft_queue.fft[fft_queue.i].clone()
        }
        rem => time_interpolate(
            &(fft_queue.fft[fft_queue.i - 1]),
            &(fft_queue.fft[fft_queue.i]),
            rem as f32 / interval as f32,
        ),
    };

    for (handle, new_value) in fft_queue.curr_bars.chunks(2).zip(curr_fft.iter()) {
        let (handle1, handle2) = (handle[0].clone(), handle[1].clone());

        let rect = meshes.get_mut(handle1).unwrap();
        let dims = rect.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = new_value.clone() * (h / 2) as f32;
                x[1][1] = new_value.clone() * (h / 2) as f32;
                x[2][1] = -new_value.clone() * (h / 2) as f32;
                x[3][1] = -new_value.clone() * (h / 2) as f32;
            }
            _ => {}
        }

        let rect = meshes.get_mut(handle2).unwrap();
        let dims = rect.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = new_value.clone() * (h / 2) as f32 - args.border_size as f32;
                x[1][1] = new_value.clone() * (h / 2) as f32 - args.border_size as f32;
                x[2][1] = -new_value.clone() * (h / 2) as f32 + args.border_size as f32;
                x[3][1] = -new_value.clone() * (h / 2) as f32 + args.border_size as f32;
            }
            _ => {}
        }
    }

    if update_i {
        fft_queue.i += 1;
    }
    fft_queue.c += 1;
}

fn startup(
    mut commands: Commands,
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut fft_queue: ResMut<FFTQueue>,
    args: Res<FFTArgs>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(
                Color::hex(args.background_color.clone()).unwrap(),
            ),
            ..Default::default()
        },
        ..Default::default()
    });

    let w = window.single_mut().physical_width();
    let h = window.single_mut().physical_height();

    if !args.disable_title {
        let font = asset_server.load("fonts/Roboto-Regular.ttf");
        let text_style = TextStyle {
            font: font.clone(),
            font_size: args.font_size as f32,
            color: Color::hex(args.text_color.clone()).unwrap(),
        };
    
        commands.spawn(
            Text2dBundle {
                text: Text::from_section(format!("Playing: \"{}\"", args.file_path.file_name().unwrap().to_str().unwrap()), text_style.clone()),
                transform: Transform::from_xyz(-(w as f32 / 2.0) + 10.0, (h as f32 / 2.0) - 10.0, 0.0),
                text_anchor: Anchor::TopLeft,
                ..default()
            },
        );
    };

    let num_bars = fft_queue.fft[0].len();
    let bar_size = w as f32 / num_bars as f32;
    let mut handle_vec = Vec::new();

    for i in 0..num_bars {
        let handle1 = meshes.add(Rectangle::new(bar_size, 0.0));
        handle_vec.push(handle1.clone());

        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(handle1),
            material: materials.add(Color::hex(args.border_color.clone()).unwrap()),
            transform: Transform::from_xyz(
                bar_size * i as f32 + (bar_size / 2.0) - (w / 2) as f32,
                0.0,
                0.0,
            ),
            ..default()
        });

        let handle2 = meshes.add(Rectangle::new(bar_size - args.border_size as f32, 0.0));
        handle_vec.push(handle2.clone());

        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(handle2),
            material: materials.add(Color::hex(args.bar_color.clone()).unwrap()),
            transform: Transform::from_xyz(
                bar_size * i as f32 + (bar_size / 2.0) - (w / 2) as f32,
                0.0,
                0.0,
            ),
            ..default()
        });
    }
    fft_queue.curr_bars = handle_vec;
}

fn main() {
    let args = cli_args_to_fft_args(CLIArgs::parse());

    let p = PathBuf::from(OsString::from(&args.file_path));

    let file_name = p.file_stem().unwrap().to_str().unwrap();
    let mut cache_path = p.parent().unwrap().to_path_buf();
    cache_path.push(format!(".{}.fft", file_name));

    println!("Computing FFT...");
    let mut fft = compute_fft(&p);

    fft = normalize_fft(fft, RESCALING_THRESHOLDS, RESCALING_FACTOR);
    let mut fft_vec = fft.fft;

    for c in fft_vec.iter_mut() {
        let mut reversed = c.clone();
        reversed.reverse();
        reversed.append(c);
        *c = reversed;
    }

    fft_vec
        .par_iter_mut()
        .for_each(|x| space_interpolate(x, BAR_INTERPOLATION_FACTOR));

    let mut binding = App::new();
    let app = binding
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "fftviz".into(),
                name: Some("fftviz".into()),
                resolution: (SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32).into(),
                resizable: false,
                position: WindowPosition::Centered(MonitorSelection::Current),
                prevent_default_event_handling: false,
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                visible: true,
                ..default()
            }),
            ..default()
        }),))
        .insert_resource(FFTQueue {
            fft: fft_vec,
            curr_bars: Vec::new(),
            i: 0,
            c: 0,
        })
        .insert_resource(args)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (update_bars.run_if(bevy::time::common_conditions::on_timer(
                Duration::from_secs_f64(1.0 / RENDERING_FPS as f64),
            )),),
        );

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open(&p).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();

    app.run();
}
