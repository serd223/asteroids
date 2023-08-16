use framebrush::{Canvas, RGBu32, BLUE, GREEN, RED, YELLOW};
use rand::{rngs::ThreadRng, Rng};
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

mod math;

const SCREEN_WIDTH: u32 = 640;
const SCREEN_HEIGHT: u32 = 576;

const CANVAS_WIDTH: usize = 160;
const CANVAS_HEIGHT: usize = 144;

const DANGER_ZONE: f32 = (1. / 4.) * (CANVAS_HEIGHT as f32);

const DEFAULT_ACCELERATION: f32 = 25.;
const DEFAULT_BULLET_COOLDOWN: u64 = 1900;
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

struct Ship {
    x: f32,
    y: f32,
    vertices: [(f32, f32); 3],
    transform_scale: f32,
    rot: f32,
    velocity: (f32, f32),
    acc: f32,
    hitbox: [(f32, f32); 4],
}

impl Ship {
    fn update(&mut self, delta_time: f32) {
        self.x += self.velocity.0 * delta_time;
        self.y += self.velocity.1 * delta_time;
        self.hitbox = {
            let mut res = self.hitbox;

            res[2] = (
                self.vertices[0].0 * self.transform_scale / 2.,
                self.vertices[0].1 * self.transform_scale / 2.,
            );
            res[3] = (self.vertices[1].0 * self.transform_scale / 2., res[2].1);

            res[0] = (res[2].0, self.vertices[2].1 * self.transform_scale / 2.);
            res[1] = (res[3].0, res[0].1);

            res.map(|(x, y)| {
                let (x, y) = rotate(x, y, self.rot);
                (x + self.x, y + self.y)
            })
        };
    }
}

struct Asteroid {
    x: f32,
    y: f32,
    vertices: [(f32, f32); 4],
    velocity: (f32, f32),
    scale: f32,
    transform: [(f32, f32); 4],
}

fn randf32(rng: &mut ThreadRng) -> f32 {
    rng.gen::<f32>() * 2. - 1.
}

impl Asteroid {
    fn random(rng: &mut ThreadRng, ship: &Ship) -> Self {
        loop {
            let velocity = (randf32(rng) * 25., randf32(rng) * 25.);
            let mut res = Self {
                x: if velocity.0 >= 0. {
                    rng.gen::<f32>() * DANGER_ZONE
                } else {
                    CANVAS_WIDTH as f32 - (rng.gen::<f32>() * DANGER_ZONE)
                },
                y: if velocity.1 >= 0. {
                    rng.gen::<f32>() * DANGER_ZONE
                } else {
                    CANVAS_HEIGHT as f32 - (rng.gen::<f32>() * DANGER_ZONE)
                },
                vertices: [
                    normalise(1. - randf32(rng), 1. - randf32(rng)),
                    normalise(1. - randf32(rng), -1. + randf32(rng)),
                    normalise(-1. + randf32(rng), -1. + randf32(rng)),
                    normalise(-1. + randf32(rng), 1. - randf32(rng)),
                ],
                velocity,
                scale: 8. * (rng.gen::<f32>() + 1.),
                transform: [(0., 0.); 4],
            };

            for (i, t) in res.transform.iter_mut().enumerate() {
                t.0 = res.vertices[i].0 * res.scale + res.x;
                t.1 = res.vertices[i].1 * res.scale + res.y;
            }
            let mut inside_ship = false;
            inside_ship |= ship.hitbox.iter().any(|(x, y)| res.contains(*x, *y));
            inside_ship |= res.contains(ship.x, ship.y);
            if !inside_ship {
                return res;
            }
        }
    }

    fn apply_transform(&mut self) {
        for (i, t) in self.transform.iter_mut().enumerate() {
            t.0 = self.vertices[i].0 * self.scale + self.x;
            t.1 = self.vertices[i].1 * self.scale + self.y;
        }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        let (mut left, mut right, mut top, mut bottom): (f32, f32, f32, f32) = (
            self.transform[0].0,
            self.transform[0].0,
            self.transform[0].1,
            self.transform[0].1,
        );

        for (x, y) in self.transform {
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
        }

        (left..right).contains(&x) && (top..bottom).contains(&y)
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut score = 0;
    let mut high_score = 0;
    let mut ship = Ship {
        x: CANVAS_WIDTH as f32 / 2.,
        y: CANVAS_HEIGHT as f32 / 2.,
        vertices: [normalise(-1., -1.), normalise(1., -1.), (0., 1.)],
        transform_scale: 10.,
        rot: 0.,
        velocity: (0., 0.),
        acc: DEFAULT_ACCELERATION,
        hitbox: [(0., 0.); 4],
    };
    ship.update(0.);

    let mut asteroids = vec![Asteroid::random(&mut rng, &ship)];
    let min_asteroid_spawn_interval = 0.5; // seconds
    let mut last_asteroid_spawn = Instant::now();

    let mut bullets: Vec<(f32, f32, (f32, f32))> = vec![];
    let mut bullet_cooldown = DEFAULT_BULLET_COOLDOWN; // milliseconds
    let mut last_bullet = Instant::now() - Duration::from_millis(bullet_cooldown);

    let mut show_hitbox = false;

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

    println!(
        r#"Welcome to..
... A S T E R O I D S ...

Controls:
    Arrow Keys to move,
    [X] to shoot,
    (Debug) [Z] to show hitbox

Tips:
    * The orange-ish zone is the "Danger Zone", asteroids only spawn in the Danger Zone.
    Try to avoid staying inside the Danger Zone or an astroid might spawn close to you (but never inside you).
    * Your weapon has a pretty long cooldown, only use it when necessary!
    * Asteroids will spawn faster as you progress.

Good luck!"#
    );
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
                let now = Instant::now();
                let frame_time = now - last_redraw;
                let min_frame_time = 17000;

                if frame_time.as_micros() > min_frame_time {
                    let delta_time = frame_time.as_secs_f32();
                    if pressed_keys[VirtualKeyCode::Z as usize]
                        && !prev_pressed_keys[VirtualKeyCode::Z as usize] {
                        show_hitbox = !show_hitbox;
                    }

                    if pressed_keys[VirtualKeyCode::Left as usize] {
                        ship.rot -= 3.5 * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Right as usize] {
                        ship.rot += 3.5 * delta_time;
                    }

                    let (s, c) = (ship.rot + FRAC_PI_2).sin_cos();

                    let mut moving = false;
                    if pressed_keys[VirtualKeyCode::Up as usize] {
                        moving = true;
                        ship.velocity.0 += c * ship.acc * delta_time;
                        ship.velocity.1 += s * ship.acc * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Down as usize] {
                        moving = true;
                        ship.velocity.0 -= c * ship.acc * delta_time;
                        ship.velocity.1 -= s * ship.acc * delta_time;
                    }

                    let min_vel = 0.75;
                    let acc_mul = 1. / 1.2;

                    if !moving {
                        if ship.velocity.0 >= min_vel {
                            ship.velocity.0 -= ship.acc * acc_mul * delta_time;
                        } else if ship.velocity.0 <= -min_vel {
                            ship.velocity.0 += ship.acc * acc_mul * delta_time;
                        } else {
                            ship.velocity.0 = 0.
                        }

                        if ship.velocity.1 >= min_vel {
                            ship.velocity.1 -= ship.acc * acc_mul * delta_time;
                        } else if ship.velocity.1 <= -min_vel {
                            ship.velocity.1 += ship.acc * acc_mul * delta_time;
                        } else {
                            ship.velocity.1 = 0.
                        }
                    }

                    ship.update(delta_time);
                    let horizontal_edge = (CANVAS_WIDTH - 1) as f32;
                    if ship.x < 0. {
                        ship.x = horizontal_edge;
                    } else if ship.x > horizontal_edge {
                        ship.x = 0.
                    }

                    let vertical_edge = (CANVAS_HEIGHT - 1) as f32;
                    if ship.y < 0. {
                        ship.y = vertical_edge;
                    } else if ship.y > vertical_edge {
                        ship.y = 0.
                    }

                    if pressed_keys[VirtualKeyCode::X as usize]
                        && !prev_pressed_keys[VirtualKeyCode::X as usize]
                        && (now - last_bullet).as_millis() as u64 >= bullet_cooldown
                    {
                        let dir = (ship.rot + FRAC_PI_2).sin_cos();
                        bullets.push((
                            ship.x + dir.1 * ship.transform_scale,
                            ship.y + dir.0 * ship.transform_scale,
                            dir,
                        ));
                        last_bullet = now;
                    }

                    bullets.retain_mut(|(x, y, dir)| {
                        *x += dir.1 * 95. * delta_time;
                        *y += dir.0 * 95. * delta_time;

                        *x >= 0.
                            && *x < CANVAS_WIDTH as f32
                            && *y >= 0.
                            && *y < CANVAS_HEIGHT as f32
                    });


                    if (now - last_asteroid_spawn).as_secs_f32() >= (min_asteroid_spawn_interval / ((score + 1) as f32 / 10.)).max(min_asteroid_spawn_interval) {
                        asteroids.push(Asteroid::random(&mut rng, &ship));
                        last_asteroid_spawn = now;
                    }

                    let mut ship_hit = false;
                    asteroids.retain_mut(|asteroid| {
                        asteroid.x += asteroid.velocity.0 * delta_time;
                        asteroid.y += asteroid.velocity.1 * delta_time;
                        asteroid.apply_transform();

                        ship_hit |= ship.hitbox.iter().any(|(x, y)| asteroid.contains(*x, *y));
                        ship_hit |= asteroid.contains(ship.x, ship.y);

                        let bullet_hit = bullets.iter_mut().any(|(x, y, _)| {
                            let res = asteroid.contains(*x, *y);
                            if res {
                                *x = (CANVAS_WIDTH * 2) as f32;
                                *y = (CANVAS_HEIGHT * 2) as f32;
                            }
                            res
                        });
                        if bullet_hit {
                            score += 1;
                            ship.acc += (score as f32) / 32.;
                            if score % 5 == 0 {
                                bullet_cooldown -= 200;
                                bullet_cooldown = bullet_cooldown.max(700);
                            }
                            // TODO remove later
                            println!("\n[Explosion Sounds] Score: {score}");
                        }

                        asteroid.x >= 0.
                            && asteroid.x < CANVAS_WIDTH as f32
                            && asteroid.y >= 0.
                            && asteroid.y < CANVAS_HEIGHT as f32
                            && !bullet_hit
                    });

                    if ship_hit {
                        high_score = high_score.max(score);
                        // TODO remove later
                        println!("\n[Ship Explosion] You crashed! Score: {score}, High Score: {high_score}");
                        asteroids.clear();
                        asteroids.push(Asteroid::random(&mut rng, &ship));
                        last_asteroid_spawn = now;
                        bullets.clear();
                        bullet_cooldown = DEFAULT_BULLET_COOLDOWN;
                        last_bullet = now - Duration::from_millis(bullet_cooldown);
                        ship.x = (CANVAS_WIDTH / 2) as f32;
                        ship.y = (CANVAS_HEIGHT / 2) as f32;
                        ship.acc = DEFAULT_ACCELERATION;
                        ship.velocity = (0., 0.);
                        ship.rot = FRAC_PI_2 * 2.;
                        score = 0;
                    }

                    last_redraw = now;
                    prev_pressed_keys = pressed_keys;
                    window.request_redraw();
                }
            }

            Event::RedrawRequested(id) if id == window.id() => {
                let (width, height) = {
                    let window_size = window.inner_size();

                    (window_size.width, window_size.height)
                };

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

                    let danger_zone_color = RGBu32::Rgb(40, 15, 0);
                    canvas.rect(0, 0, CANVAS_WIDTH, DANGER_ZONE as usize, &danger_zone_color);
                    canvas.rect(0, CANVAS_HEIGHT - DANGER_ZONE as usize, CANVAS_WIDTH, DANGER_ZONE as usize, &danger_zone_color);
                    canvas.rect(0, 0, DANGER_ZONE as usize, CANVAS_HEIGHT, &danger_zone_color);
                    canvas.rect(CANVAS_WIDTH - DANGER_ZONE as usize, 0, DANGER_ZONE as usize, CANVAS_HEIGHT, &danger_zone_color);

                    for asteroid in &asteroids {
                        for (i, v) in asteroid.transform.iter().enumerate() {
                            if i > 0 {
                                canvas.line(
                                    v.0 as usize,
                                    v.1 as usize,
                                    asteroid.transform[i - 1].0 as usize,
                                    asteroid.transform[i - 1].1 as usize,
                                    &GREEN,
                                )
                            } else {
                                let len = asteroid.transform.len();
                                canvas.line(
                                    v.0 as usize,
                                    v.1 as usize,
                                    asteroid.transform[len - 1].0 as usize,
                                    asteroid.transform[len - 1].1 as usize,
                                    &GREEN,
                                )
                            }
                        }
                    }

                    let ship_transform = ship.vertices.map(|(x, y)| {
                        let (x, y) = rotate(x, y, ship.rot);

                        let (x, y) = (
                            x * ship.transform_scale + ship.x,
                            y * ship.transform_scale + ship.y,
                        );
                        // (x as usize, y as usize)
                        (x as usize, y as usize)
                    });
                    for (x0, y0) in ship_transform {
                        for (x1, y1) in ship_transform {
                            canvas.line(x0, y0, x1, y1, &RED);
                        }
                    }

                    for (x, y, _) in &bullets {
                        canvas.put(*x as usize, *y as usize, &BLUE)
                    }

                    if show_hitbox {
                        for (x0, y0) in ship.hitbox {
                            for (x1, y1) in ship.hitbox {
                                canvas.line(
                                    x0 as usize,
                                    y0 as usize,
                                    x1 as usize,
                                    y1 as usize,
                                    &YELLOW,
                                );
                            }
                        }
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
