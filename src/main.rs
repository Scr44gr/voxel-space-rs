// Voxel Space implementation in Rust.
use minifb::{Key, Window, WindowOptions};
mod utils;
use crate::utils::from_rgb8_to_u32;

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;
const SCALE_FACTOR: f32 = 100.0;
const MAP_SIZE: i32 = 1024;

struct Camera {
    pub x: f32,
    pub y: f32,
    pub zfar: f32,
    pub angle: f32,
    pub height: f32,
    pub horizon: f32,
}

fn clear_buffer(buffer: &mut Vec<u32>) {
    for i in 0..buffer.len() {
        buffer[i] = from_rgb8_to_u32(135, 206, 235);
    }
}

fn handle_input(window: &Window, camera: &mut Camera) {
    let mouse_pos = window.get_mouse_pos(minifb::MouseMode::Clamp).unwrap();
    // camera angle
    if window.is_key_down(Key::D) {
        camera.angle += 0.04;
    }

    if window.is_key_down(Key::A) {
        camera.angle -= 0.04;
    }

    if window.is_key_down(Key::S) {
        camera.y -= camera.angle.sin();
        camera.x -= camera.angle.cos();
    }

    if window.is_key_down(Key::W) {
        camera.y += camera.angle.sin();
        camera.x += camera.angle.cos();
    }

    // camera height
    if window.is_key_down(Key::E) {
        camera.height += 4.0;
    }

    if window.is_key_down(Key::Q) {
        camera.height -= 4.0;
    }
    camera.horizon = mouse_pos.1 as f32 / 4.0;
    camera.angle = (mouse_pos.0.to_radians() / 4.0) as f32;
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

    let mut window = Window::new(
        "Voxel Space Impl - ESC to exit",
        800,
        600,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);
    // hide mouse cursor
    //window.set_cursor_visibility(false);
    // set cursor position to center
    window.set_cursor_style(minifb::CursorStyle::Crosshair);

    let mut camera = Camera {
        x: 512.0,
        y: 512.0,
        zfar: 400.0,
        angle: 0.0,
        height: 1.0,
        horizon: 1.0,
    };

    let colormap = image::open("./assets/map0.color.gif")
        .unwrap()
        .to_rgb8()
        .to_vec();
    let heightmap = image::open("./assets/map0.height.gif")
        .unwrap()
        .to_luma8()
        .to_vec();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        clear_buffer(&mut buffer);
        handle_input(&window, &mut camera);

        let cosangle = camera.angle.cos();
        let sinangle = camera.angle.sin();

        let plx = cosangle * camera.zfar.abs() + sinangle * camera.zfar.abs();
        let ply = sinangle * camera.zfar.abs() - cosangle * camera.zfar.abs();

        let prx = cosangle * camera.zfar.abs() - sinangle * camera.zfar.abs();
        let pry = sinangle * camera.zfar.abs() + cosangle * camera.zfar.abs();

        for i in 0..SCREEN_WIDTH {
            let delta_x = (plx + (prx - plx) / SCREEN_WIDTH as f32 * i as f32) / camera.zfar;
            let delta_y = (ply + (pry - ply) / SCREEN_WIDTH as f32 * i as f32) / camera.zfar;

            let mut rx = camera.x;
            let mut ry = camera.y;
            let mut max_height: i32 = SCREEN_HEIGHT as i32;

            for z in 1..camera.zfar as i32 {
                rx += delta_x;
                ry += delta_y;

                let mapoffset = (MAP_SIZE as i32 * (ry as i32 & (MAP_SIZE - 1))
                    + (rx as i32 & (MAP_SIZE - 1))) as usize;

                let heightonscreen = (((camera.height) - (heightmap[mapoffset]) as f32) / z as f32
                    * SCALE_FACTOR
                    + camera.horizon) as i32;

                if heightonscreen < max_height {
                    // Draw Pixels from previous max height to current height
                    // for y = heightonscreen; y < max_height; y++
                    for y in heightonscreen..max_height {
                        if y < 0 || y >= SCREEN_HEIGHT as i32 {
                            continue;
                        }
                        buffer[(SCREEN_WIDTH * y as usize) + i] = from_rgb8_to_u32(
                            colormap[mapoffset * 3],
                            colormap[mapoffset * 3 + 1],
                            colormap[mapoffset * 3 + 2],
                        );
                    }
                    max_height = heightonscreen;
                }
            }
        }
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
