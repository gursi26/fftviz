use crate::fft::time_interpolate;
use crate::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::prelude::*;

pub fn update_fft(
    mut window: Query<&mut Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    fft_state: ResMut<FFTState>,
    app_state: ResMut<AppState>,
    args: ResMut<FFTArgs>,
) {
    let h = window.single_mut().height();
    let interval = app_state.rendering_fps / app_state.fft_fps;

    // Get the current frame (either from fft or interpolation)
    let curr_fft = match fft_state.total_frame_counter as u32 % interval {
        0 => {
            if fft_state.fft_frame_counter >= fft_state.fft.len() - 1 {
                std::process::exit(0);
            }
            fft_state.fft[fft_state.fft_frame_counter].clone()
        }
        rem => time_interpolate(
            &(fft_state.fft[fft_state.fft_frame_counter]),
            &(fft_state.fft[fft_state.fft_frame_counter + 1]),
            rem as f32 / interval as f32,
        ),
    };

    // Iterate through all currently displayed bars to change values
    for (handle, new_value) in fft_state.curr_bars.chunks(2).zip(curr_fft.iter()) {
        let (handle1, handle2) = (handle[0].0.clone_weak(), handle[1].0.clone_weak());

        let dims = meshes
            .get_mut(handle1)
            .unwrap()
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .unwrap();
        let bar_value_1 = (new_value.clone() * (h / 2.0) as f32).clamp(
            h * MIN_BAR_HEIGHT + args.border_size as f32,
            h * MAX_BAR_HEIGHT + args.border_size as f32,
        );
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = bar_value_1;
                x[1][1] = bar_value_1;
                x[2][1] = -bar_value_1;
                x[3][1] = -bar_value_1;
            }
            _ => {}
        }

        let dims = meshes
            .get_mut(handle2)
            .unwrap()
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .unwrap();
        let bar_value_2 = (new_value.clone() * (h / 2.0) as f32 - args.border_size as f32)
            .clamp(h * MIN_BAR_HEIGHT, h * MAX_BAR_HEIGHT);
        match dims {
            VertexAttributeValues::Float32x3(x) => {
                x[0][1] = bar_value_2;
                x[1][1] = bar_value_2;
                x[2][1] = -bar_value_2;
                x[3][1] = -bar_value_2;
            }
            _ => {}
        }
    }
}
