//! The classic table tennis–themed video game.
use std::fmt;

use macroquad::prelude::*;

const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;

const BACKGROUND_COLOR: Color = DARKGRAY;
const FOREGROUND_COLOR: Color = WHITE;

const RACKET_SIZE: (f32, f32) = (20., 100.);
const RACKET_MARGIN: f32 = 40.;
const RACKET_SPEED: f32 = 500.;

const BALL_SIZE: f32 = 20.;
const BALL_INIT_SPEED: f32 = 150.;
const BALL_ACCEL: f32 = 10.;

const WIN_SCORE: i32 = 5;

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Left => write!(f, "LEFT"),
            Side::Right => write!(f, "RIGHT"),
        }
    }
}

struct Racket {
    side: Side,
    pos: (f32, f32),
}

impl Racket {
    fn new(side: Side) -> Self {
        let pos_x = match side {
            Side::Left => RACKET_MARGIN,
            Side::Right => WINDOW_WIDTH - RACKET_MARGIN - RACKET_SIZE.0,
        };
        let pos_y = WINDOW_HEIGHT * 0.5 - RACKET_SIZE.1 * 0.5;
        Self {
            side,
            pos: (pos_x, pos_y),
        }
    }

    fn slide(&mut self, speed: f32) {
        let pos_y = self.pos.1 + speed * get_frame_time();
        self.pos.1 = if pos_y < 0. {
            0.
        } else if pos_y + RACKET_SIZE.1 > WINDOW_HEIGHT {
            WINDOW_HEIGHT - RACKET_SIZE.1
        } else {
            pos_y
        };
    }

    fn draw(&self) {
        draw_rectangle(
            self.pos.0,
            self.pos.1,
            RACKET_SIZE.0,
            RACKET_SIZE.1,
            FOREGROUND_COLOR,
        );
    }
}

struct Ball {
    pos: (f32, f32),
    dir: (f32, f32),
    speed: f32,
}

impl Ball {
    fn new(side: Option<Side>) -> Self {
        let x = WINDOW_WIDTH * 0.5 - BALL_SIZE * 0.5;
        let y = WINDOW_HEIGHT * 0.5 - BALL_SIZE * 0.5;
        let rnddir = || -> f32 { ((((get_time() * 1e6) as i32) & 1) * 2 - 1) as f32 };
        let dir_x = if let Some(side) = side {
            match side {
                Side::Left => -1.,
                Side::Right => 1.,
            }
        } else {
            rnddir()
        };
        Self {
            pos: (x, y),
            dir: (dir_x, rnddir()),
            speed: BALL_INIT_SPEED,
        }
    }

    fn update(&mut self) {
        let ft = get_frame_time();

        let delta = self.speed * ft;

        self.pos.0 += self.dir.0 * delta;

        let y = self.pos.1 + self.dir.1 * delta;
        (self.pos.1, self.dir.1) = if y < 0. {
            (0., -self.dir.1)
        } else if y + BALL_SIZE > WINDOW_HEIGHT {
            (WINDOW_HEIGHT - BALL_SIZE, -self.dir.1)
        } else {
            (y, self.dir.1)
        };

        self.speed += ft * BALL_ACCEL;
    }

    fn draw(&self) {
        draw_rectangle(
            self.pos.0,
            self.pos.1,
            BALL_SIZE,
            BALL_SIZE,
            FOREGROUND_COLOR,
        );
    }
}

#[derive(Clone, Copy)]
enum PongState {
    NewRound(Side),
    Playing,
    Winner(Side),
    Exit,
}

struct Pong {
    rackets: (Racket, Racket),
    scores: (i32, i32),
    ball: Ball,
    state: PongState,
}

impl Pong {
    fn new() -> Self {
        Self {
            rackets: (Racket::new(Side::Left), Racket::new(Side::Right)),
            ball: Ball::new(None),
            scores: (0, 0),
            state: PongState::Playing,
        }
    }

    fn reset(&mut self) {
        *self = Pong::new();
    }

    fn update_scores(&mut self) {
        if self.ball.pos.0 < 0. {
            self.scores.1 += 1;
            self.state = if self.scores.1 >= WIN_SCORE {
                PongState::Winner(Side::Right)
            } else {
                PongState::NewRound(Side::Left)
            };
        } else if self.ball.pos.0 + BALL_SIZE > WINDOW_WIDTH {
            self.scores.0 += 1;
            self.state = if self.scores.0 >= WIN_SCORE {
                PongState::Winner(Side::Left)
            } else {
                PongState::NewRound(Side::Right)
            };
        }
    }

    fn update_collisions(&mut self) {
        for racket in [&self.rackets.0, &self.rackets.1] {
            let ball_rect = Rect::new(self.ball.pos.0, self.ball.pos.1, BALL_SIZE, BALL_SIZE);
            let racket_rect = Rect::new(racket.pos.0, racket.pos.1, RACKET_SIZE.0, RACKET_SIZE.1);

            let Some(rect) = racket_rect.intersect(ball_rect) else {
                continue;
            };

            let racket_rect_cx = racket_rect.x + racket_rect.w * 0.5;

            match racket.side {
                Side::Left => {
                    if rect.x < racket_rect_cx {
                        continue;
                    }
                    self.ball.dir.0 = self.ball.dir.0.abs();
                }
                Side::Right => {
                    if rect.x + rect.w > racket_rect_cx {
                        continue;
                    }
                    self.ball.dir.0 = -self.ball.dir.0.abs();
                }
            }

            let rect_cy = rect.y + rect.h * 0.5;
            let racket_rect_cy = racket_rect.y + racket_rect.h * 0.5;
            let ball_dir = (rect_cy - racket_rect_cy) / (racket_rect.h * 0.5);
            self.ball.dir.1 = ball_dir;
        }
    }

    fn update(&mut self) {
        if is_key_down(KeyCode::Q) {
            self.state = PongState::Exit
        }

        match self.state {
            PongState::NewRound(side) => {
                self.ball = Ball::new(Some(side));
                self.state = PongState::Playing;
            }
            PongState::Playing => {
                if is_key_down(KeyCode::W) {
                    self.rackets.0.slide(-RACKET_SPEED);
                }
                if is_key_down(KeyCode::S) {
                    self.rackets.0.slide(RACKET_SPEED);
                }

                if is_key_down(KeyCode::Up) {
                    self.rackets.1.slide(-RACKET_SPEED);
                }
                if is_key_down(KeyCode::Down) {
                    self.rackets.1.slide(RACKET_SPEED);
                }
                self.ball.update();
                self.update_collisions();
                self.update_scores();
            }
            PongState::Winner(_) => {
                if is_key_down(KeyCode::Space) {
                    self.reset();
                }
            }
            PongState::Exit => {}
        }
    }

    fn draw_scores(&self) {
        draw_text_center(
            &format!("{} - {}", self.scores.0, self.scores.1),
            75.0,
            30.0,
        );
    }

    fn draw_winner(&self, side: Side) {
        draw_text_center(&format!("{side} WON!"), 150.0, WINDOW_HEIGHT * 0.5);
        draw_text_center(
            "(Press SPACE to play again)",
            40.,
            WINDOW_HEIGHT * 0.5 + 100.,
        );
    }

    fn draw(&self) {
        match self.state {
            PongState::Playing | PongState::NewRound(_) => {
                self.draw_scores();
                self.rackets.0.draw();
                self.rackets.1.draw();
                self.ball.draw();
            }
            PongState::Winner(side) => self.draw_winner(side),
            PongState::Exit => {}
        }
    }

    fn state(&self) -> PongState {
        self.state
    }
}

#[cfg(debug_assertions)]
fn draw_fps() {
    let fps = format!("{:3} FPS", get_fps());
    draw_text(&fps, 10., 20., 20., GREEN);
}

fn draw_text_center(text: &str, font_size: f32, y: f32) {
    let text_sz = measure_text(text, None, font_size as u16, 1.);
    draw_text(
        text,
        WINDOW_WIDTH * 0.5 - text_sz.width * 0.5,
        y - text_sz.height * 0.5 + text_sz.offset_y,
        font_size,
        FOREGROUND_COLOR,
    );
}

fn window_conf() -> Conf {
    Conf {
        window_title: "PONG".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let render_target = render_target(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);
    let mut render_camera =
        Camera2D::from_display_rect(Rect::new(0., 0., WINDOW_WIDTH, WINDOW_HEIGHT));
    render_camera.render_target = Some(render_target.clone());

    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        Default::default(),
    )
    .unwrap();

    let mut pong = Pong::new();

    loop {
        set_camera(&render_camera);

        clear_background(BACKGROUND_COLOR);

        pong.update();
        if matches!(pong.state(), PongState::Exit) {
            break;
        }
        pong.draw();

        set_default_camera();

        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();

        #[cfg(debug_assertions)]
        draw_fps();

        next_frame().await;
    }
}

// Shaders based on https://www.shadertoy.com/view/XtlSD7

const VERTEX_SHADER: &str = r#"
#version 100

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 100

precision lowp float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D Texture;
uniform vec4 _Time;

vec2 crt_curve_uv(vec2 uv) {
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs(uv.yx) / vec2(6.0, 4.0);
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void draw_vignette(inout vec3 color, vec2 uv) {
    float vignette = uv.x * uv.y * (1.0 - uv.x) * (1.0 - uv.y);
    vignette = clamp(pow(16.0 * vignette, 0.3), 0.0, 1.0);
    color *= vignette;
}

void draw_scanline(inout vec3 color, vec2 uv) {
    float scanline = clamp(0.95 + 0.05 * cos(3.14 * (uv.y + 0.008 * _Time.x) * 240.0 * 1.0), 0.0, 1.0);
    float grille = 0.85 + 0.15 * clamp(1.5 * cos(3.14 * uv.x * 640.0 * 1.0), 0.0, 1.0);
    color *= scanline * grille * 1.2;
}

void main() {
    vec3 frag_color = texture2D(Texture, uv).rgb * color.rgb;
    vec2 crt_uv = crt_curve_uv(uv);
    if (crt_uv.x < 0.0 || crt_uv.x > 1.0 || crt_uv.y < 0.0 || crt_uv.y > 1.0) {
        frag_color = vec3(0.0, 0.0, 0.0);
    }
    draw_vignette(frag_color, crt_uv);
    draw_scanline(frag_color, uv);
    gl_FragColor = vec4(frag_color, 1.0);
}
"#;
