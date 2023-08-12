use framebrush::{Canvas, BLUE, RED};
use std::{
    f32::consts::FRAC_PI_2,
    num::NonZeroU32,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 576;

const CANVAS_WIDTH: usize = 160;
const CANVAS_HEIGHT: usize = 144;

/// cos(x + y) = cos x * cos y - sin x * sin y
/// sin(x + y) = sin x * cos y + cos x * sin y
pub fn rotate(x: f32, y: f32, rot: f32) -> (f32, f32) {
    let (s, c) = rot.sin_cos();
    (x * c - y * s, y * c + x * s)
}

pub fn normalise(x: f32, y: f32) -> (f32, f32) {
    let len = (x * x + y * y).sqrt();

    (x / len, y / len)
}

fn main() {
    let triangle_vertices = [normalise(-1., -1.), normalise(1., -1.), (0., 1.)];
    let mut rot = 0.;
    let (tri_x, tri_y) = (CANVAS_WIDTH / 2, CANVAS_HEIGHT / 2);
    let (mut tri_x, mut tri_y) = (tri_x as f32, tri_y as f32);
    let mut bullets: Vec<(f32, f32, (f32, f32))> = vec![];
    let tri_transform_scale = 10.;

    let mut last_redraw = Instant::now();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
        .with_title("Asteroids")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { softbuffer::Context::new(&window) }.unwrap();
    let mut surface = unsafe { softbuffer::Surface::new(&context, &window) }.unwrap();
    let mut pressed_keys = [false; 256];
    let mut prev_pressed_keys = [false; 256];

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            Instant::now()
                .checked_add(Duration::from_micros(1_000_000 / 144))
                .unwrap(),
        );

        match event {
            Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
            } if window_id == window.id() => match state {
                ElementState::Pressed => pressed_keys[keycode as usize] = true,
                ElementState::Released => pressed_keys[keycode as usize] = false,
            },

            Event::MainEventsCleared => {
                let current_time = Instant::now();
                let frame_time = current_time - last_redraw;
                let min_frame_time = 17000;

                if frame_time.as_micros() > min_frame_time {
                    let delta_time = frame_time.as_secs_f32();

                    if pressed_keys[VirtualKeyCode::Left as usize] {
                        rot -= 2.5 * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Right as usize] {
                        rot += 2.5 * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Up as usize] {
                        let (s, c) = (rot + FRAC_PI_2).sin_cos();
                        tri_x += c * 25. * delta_time;
                        tri_y += s * 25. * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Down as usize] {
                        let (s, c) = (rot + FRAC_PI_2).sin_cos();
                        tri_x -= c * 25. * delta_time;
                        tri_y -= s * 25. * delta_time;
                    }

                    if pressed_keys[VirtualKeyCode::X as usize]
                        && !prev_pressed_keys[VirtualKeyCode::X as usize]
                    {
                        let dir = (rot + FRAC_PI_2).sin_cos();
                        bullets.push((
                            tri_x + dir.1 * tri_transform_scale,
                            tri_y + dir.0 * tri_transform_scale,
                            dir,
                        ));
                    }

                    bullets.retain_mut(|(x, y, dir)| {
                        *x = *x + dir.1 * 45. * delta_time;
                        *y = *y + dir.0 * 45. * delta_time;

                        *x >= 0.
                            && *x < CANVAS_WIDTH as f32
                            && *y >= 0.
                            && *y < CANVAS_HEIGHT as f32
                    });

                    last_redraw = current_time;
                    prev_pressed_keys = pressed_keys;
                    window.request_redraw();
                }
            }

            Event::RedrawRequested(id) if id == window.id() => {
                let (width, height) = {
                    let window_size = window.inner_size();

                    (window_size.width, window_size.height)
                };

                let transform = triangle_vertices.map(|(x, y)| {
                    let (x, y) = rotate(x, y, rot);

                    let (x, y) = (
                        x * tri_transform_scale + tri_x,
                        y * tri_transform_scale + tri_y,
                    );
                    (x as usize, y as usize)
                });

                if let (Some(width_nonzero), Some(height_nonzero)) =
                    (NonZeroU32::new(width), NonZeroU32::new(height))
                {
                    surface.resize(width_nonzero, height_nonzero).unwrap();
                    let mut buffer = surface.buffer_mut().unwrap();
                    let mut canvas = Canvas::new(
                        &mut buffer,
                        (width as usize, height as usize),
                        (CANVAS_WIDTH, CANVAS_HEIGHT),
                    );
                    canvas.fill(0);

                    for (x0, y0) in transform {
                        for (x1, y1) in transform {
                            canvas.line(x0, y0, x1, y1, &RED);
                        }
                    }

                    for (x, y, _) in &bullets {
                        canvas.put(*x as usize, *y as usize, &BLUE)
                    }
                    buffer.present().expect("Couldn't present frame buffer.");
                }
            }

            Event::WindowEvent {
                window_id: id,
                event: WindowEvent::CloseRequested,
            } if id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }

            _ => (),
        }
    });
}
