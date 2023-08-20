use framebrush::{Canvas, RGBu32, GREEN, RED, YELLOW};
use math::{vec2, Transform, Vec2};
use rand::{rngs::ThreadRng, Rng};
use std::{
    f32::consts::{FRAC_PI_2, PI},
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

// Gameboy Resoultion * 2
const CANVAS_WIDTH: usize = 320;
const CANVAS_HEIGHT: usize = 288;

// Gameboy Resolution
// const CANVAS_WIDTH: usize = 160;
// const CANVAS_HEIGHT: usize = 144;

const DANGER_ZONE: f32 = (1. / 20.) * (CANVAS_HEIGHT as f32);
const DEFAULT_ACCELERATION: f32 = 25.;
const DEFAULT_BULLET_COOLDOWN: u64 = 1100;
const BULLET_COLOR: RGBu32 = RGBu32::Rgb(86, 182, 194);

struct Ship {
    transform: Transform<3>,
    velocity: Vec2,
    acc: f32,
    hitbox: [Vec2; 4],
}

impl Ship {
    fn update(&mut self, delta_time: f32) {
        self.transform.pos.x += self.velocity.x * delta_time;
        self.transform.pos.y += self.velocity.y * delta_time;

        self.hitbox[2] = self.transform.vertices[0].clone() * (self.transform.scale / 2.);

        self.hitbox[3] = vec2(
            self.transform.vertices[1].x * self.transform.scale / 2.,
            self.hitbox[2].y,
        );

        self.hitbox[0] = vec2(
            self.hitbox[2].x,
            self.transform.vertices[2].y * self.transform.scale / 2.,
        );

        self.hitbox[1] = vec2(self.hitbox[3].x, self.hitbox[0].y);

        for v in self.hitbox.iter_mut() {
            v.rotate_mut(self.transform.rot);
            *v += &self.transform.pos;
        }
    }
}

struct Asteroid {
    transform: Transform<4>,
    velocity: Vec2,
}

fn randf32(rng: &mut ThreadRng) -> f32 {
    rng.gen::<f32>() * 2. - 1.
}

impl Asteroid {
    fn random(rng: &mut ThreadRng, ship: &Ship) -> Self {
        loop {
            let velocity = vec2(randf32(rng) * 25., randf32(rng) * 25.);
            let mut res = Self {
                transform: Transform {
                    rot: 0.,
                    pos: Vec2 {
                        x: if velocity.x >= 0. {
                            rng.gen::<f32>() * DANGER_ZONE
                        } else {
                            CANVAS_WIDTH as f32 - (rng.gen::<f32>() * DANGER_ZONE)
                        },
                        y: if velocity.y >= 0. {
                            rng.gen::<f32>() * DANGER_ZONE
                        } else {
                            CANVAS_HEIGHT as f32 - (rng.gen::<f32>() * DANGER_ZONE)
                        },
                    },
                    vertices: [
                        vec2(1. - randf32(rng), 1. - randf32(rng)).normalise(),
                        vec2(1. - randf32(rng), -1. + randf32(rng)).normalise(),
                        vec2(-1. + randf32(rng), -1. + randf32(rng)).normalise(),
                        vec2(-1. + randf32(rng), 1. - randf32(rng)).normalise(),
                    ],
                    scale: 8. * (rng.gen::<f32>() + 1.),
                    transform: [Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, Vec2::ZERO],
                },

                velocity,
            };

            res.transform.apply();

            let mut inside_ship = false;
            inside_ship |= ship.hitbox.iter().any(|Vec2 { x, y }| res.contains(*x, *y));
            inside_ship |= res.contains(ship.transform.pos.x, ship.transform.pos.y);
            if !inside_ship {
                return res;
            }
        }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        let (mut left, mut right, mut top, mut bottom): (f32, f32, f32, f32) = (
            self.transform.transform[0].x,
            self.transform.transform[0].x,
            self.transform.transform[0].y,
            self.transform.transform[0].y,
        );

        for &Vec2 { x, y } in self.transform.transform.iter() {
            left = left.min(x);
            right = right.max(x);
            top = top.min(y);
            bottom = bottom.max(y);
        }

        (left..right).contains(&x) && (top..bottom).contains(&y)
    }
}

struct Bullet {
    pos: Vec2,
    dir: Vec2,
    wrap_count: u8
}

impl Bullet {
    pub fn new(pos: Vec2, dir: Vec2) -> Self {
        Self { pos, dir, wrap_count: 0 }
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    let mut score = 0;
    let mut high_score = 0;
    let vertices = [
        vec2(-1., -1.).normalise(),
        vec2(1., -1.).normalise(),
        vec2(0., 1.),
    ];
    let mut ship = Ship {
        transform: Transform {
            pos: vec2(CANVAS_WIDTH as f32 / 2., CANVAS_HEIGHT as f32 / 2.),
            vertices: vertices.clone(),
            scale: 10.,
            rot: 0.,
            transform: vertices,
        },
        velocity: vec2(0., 0.),
        acc: DEFAULT_ACCELERATION,
        hitbox: [vec2(0., 0.), vec2(0., 0.), vec2(0., 0.), vec2(0., 0.)],
    };
    ship.update(0.);

    let mut asteroids = vec![Asteroid::random(&mut rng, &ship)];

    let mut bullets: Vec<Bullet> = vec![];
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
    * Your weapon has a cooldown but the cooldown will decrease as you progress!

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
                        ship.transform.rot -= 3.5 * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Right as usize] {
                        ship.transform.rot += 3.5 * delta_time;
                    }

                    let (s, c) = (ship.transform.rot + FRAC_PI_2).sin_cos();

                    let mut moving = false;
                    if pressed_keys[VirtualKeyCode::Up as usize] {
                        moving = true;
                        ship.velocity.x += c * ship.acc * delta_time;
                        ship.velocity.y += s * ship.acc * delta_time;
                    }
                    if pressed_keys[VirtualKeyCode::Down as usize] {
                        moving = true;
                        ship.velocity.x -= c * ship.acc * delta_time;
                        ship.velocity.y -= s * ship.acc * delta_time;
                    }

                    let min_vel = 0.75;
                    let acc_mul = 1. / 1.2;

                    if !moving {
                        if ship.velocity.x >= min_vel {
                            ship.velocity.x -= ship.acc * acc_mul * delta_time;
                        } else if ship.velocity.x <= -min_vel {
                            ship.velocity.x += ship.acc * acc_mul * delta_time;
                        } else {
                            ship.velocity.x = 0.
                        }

                        if ship.velocity.y >= min_vel {
                            ship.velocity.y -= ship.acc * acc_mul * delta_time;
                        } else if ship.velocity.y <= -min_vel {
                            ship.velocity.y += ship.acc * acc_mul * delta_time;
                        } else {
                            ship.velocity.y = 0.
                        }
                    }

                    ship.update(delta_time);
                    let horizontal_edge = (CANVAS_WIDTH - 1) as f32;
                    if ship.transform.pos.x < 0. {
                        ship.transform.pos.x = horizontal_edge;
                    } else if ship.transform.pos.x > horizontal_edge {
                        ship.transform.pos.x = 0.
                    }

                    let vertical_edge = (CANVAS_HEIGHT - 1) as f32;
                    if ship.transform.pos.y < 0. {
                        ship.transform.pos.y = vertical_edge;
                    } else if ship.transform.pos.y > vertical_edge {
                        ship.transform.pos.y = 0.
                    }

                    if pressed_keys[VirtualKeyCode::X as usize]
                        && !prev_pressed_keys[VirtualKeyCode::X as usize]
                        && (now - last_bullet).as_millis() as u64 >= bullet_cooldown
                    {
                        let dir = (ship.transform.rot + FRAC_PI_2).sin_cos();
                        let dir = vec2(dir.0, dir.1);
                        bullets.push(Bullet::new(vec2(
                            ship.transform.pos.x + dir.y * ship.transform.scale,
                            ship.transform.pos.y + dir.x * ship.transform.scale,
                        ), dir));
                        last_bullet = now;
                    }

                    bullets.retain_mut(|b| {
                        b.pos.x += b.dir.y * 155. * delta_time;
                        b.pos.y += b.dir.x * 155. * delta_time;
                        if b.pos.x < 0. {
                            b.pos.x = CANVAS_WIDTH as f32;
                            b.wrap_count += 1;
                        } else if b.pos.x > CANVAS_WIDTH as f32 {
                            b.pos.x = 0.;
                            b.wrap_count += 1;
                        }
                        if b.pos.y < 0. {
                            b.pos.y = CANVAS_HEIGHT as f32;
                            b.wrap_count += 1;
                        } else if b.pos.y > CANVAS_HEIGHT as f32 {
                            b.pos.y = 0.;
                            b.wrap_count += 1;
                        }

                        b.wrap_count < 5
                    });

                    if asteroids.len() == 0 {
                        for _ in 0..4 {
                            asteroids.push(Asteroid::random(&mut rng, &ship))
                        }
                    }

                    let mut ship_hit = false;
                    let mut new_asteroids = vec![];
                    asteroids.retain_mut(|asteroid| {
                        asteroid.transform.pos.x += asteroid.velocity.x * delta_time;
                        asteroid.transform.pos.y += asteroid.velocity.y * delta_time;
                        asteroid.transform.apply();

                        ship_hit |= ship.hitbox.iter().any(| Vec2 {x, y} | asteroid.contains(*x, *y));
                        ship_hit |= asteroid.contains(ship.transform.pos.x, ship.transform.pos.y);

                        let mut hit_index = 0;

                        let bullet_hit = bullets.iter_mut().enumerate().any(|(i, b)| {
                            let res = asteroid.contains(b.pos.x, b.pos.y);
                            if res {
                                hit_index = i;
                            }
                            res
                        });

                        if bullet_hit {
                            bullets.swap_remove(hit_index);
                            score += 1;
                            ship.acc += (score as f32) / 32.;
                            if score % 5 == 0 {
                                bullet_cooldown -= 200;
                                bullet_cooldown = bullet_cooldown.max(700);
                            }
                            // TODO remove later
                            println!("\n[Explosion Sounds] Score: {score}");

                            let n = rng.gen_range(1..=3);
                            for _ in 0..n {
                                new_asteroids.push(Asteroid {
                                    transform: Transform {
                                        scale: asteroid.transform.scale / ((rng.gen::<f32>() * 2.) + 1.),
                                        rot: randf32(&mut rng) * PI * 2.,
                                        ..asteroid.transform.clone()
                                    },
                                    velocity: vec2(randf32(&mut rng), randf32(&mut rng)).normalise() * 25.,
                                });
                            }
                        }

                        if asteroid.transform.pos.x < 0. {
                            asteroid.transform.pos.x = CANVAS_WIDTH as f32
                        } else if asteroid.transform.pos.x > CANVAS_WIDTH as f32 {
                            asteroid.transform.pos.x = 0.
                        }
                        if asteroid.transform.pos.y < 0. {
                            asteroid.transform.pos.y = CANVAS_HEIGHT as f32
                        } else if asteroid.transform.pos.y > CANVAS_HEIGHT as f32 {
                            asteroid.transform.pos.y = 0.
                        }
                        
                        !bullet_hit && asteroid.transform.scale > 3.
                    });

                    asteroids.extend(new_asteroids);

                    if ship_hit {
                        high_score = high_score.max(score);
                        // TODO remove later
                        println!("\n[Ship Explosion] You crashed! Score: {score}, High Score: {high_score}");
                        asteroids.clear();
                        asteroids.push(Asteroid::random(&mut rng, &ship));
                        bullets.clear();
                        bullet_cooldown = DEFAULT_BULLET_COOLDOWN;
                        last_bullet = now - Duration::from_millis(bullet_cooldown);
                        ship.transform.pos.x = (CANVAS_WIDTH / 2) as f32;
                        ship.transform.pos.y = (CANVAS_HEIGHT / 2) as f32;
                        ship.acc = DEFAULT_ACCELERATION;
                        ship.velocity = vec2(0., 0.);
                        ship.transform.rot = FRAC_PI_2 * 2.;
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
                        for (i, v) in asteroid.transform.transform.iter().enumerate() {
                            if i > 0 {
                                canvas.line(
                                    v.x as usize,
                                    v.y as usize,
                                    asteroid.transform.transform[i - 1].x as usize,
                                    asteroid.transform.transform[i - 1].y as usize,
                                    &GREEN,
                                )
                            } else {
                                let len = asteroid.transform.transform.len();
                                canvas.line(
                                    v.x as usize,
                                    v.y as usize,
                                    asteroid.transform.transform[len - 1].x as usize,
                                    asteroid.transform.transform[len - 1].y as usize,
                                    &GREEN,
                                )
                            }
                        }
                    }
                    ship.transform.apply();
                    for &Vec2 { x: x0, y: y0 } in ship.transform.transform.iter() {
                        for &Vec2 { x: x1, y: y1 } in ship.transform.transform.iter() {
                            canvas.line(x0 as usize, y0 as usize, x1 as usize, y1 as usize, &RED);
                        }
                    }

                    for Bullet { pos, .. } in bullets.iter() {
                        canvas.put(pos.x as usize, pos.y as usize, &BULLET_COLOR)
                    }

                    if show_hitbox {
                        for &Vec2 { x: x0, y: y0 } in ship.hitbox.iter() {
                            for &Vec2 { x: x1, y: y1 } in ship.hitbox.iter() {
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
