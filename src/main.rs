//! The classic table tennis–themed video game.
use std::fmt;

use macroquad::{
    audio::{load_sound_from_bytes, play_sound_once, Sound},
    prelude::*,
};

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
const WIN_SCREEN_SECS: f64 = 1.;

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn toggle(self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
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
        self.pos.1 += speed * get_frame_time();
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

    fn fly(&mut self) {
        let ft = get_frame_time();
        let delta = self.speed * ft;
        self.pos.0 += self.dir.0 * delta;
        self.pos.1 += self.dir.1 * delta;
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
    WallBounce,
    RacketBounce,
    Point(Side),
    Winner(Side, f64),
    Exit,
}

#[derive(Clone, Copy, PartialEq)]
enum Input {
    Up(Side),
    Down(Side),
    Quit,
    Unknown,
}

struct Pong {
    rackets: (Racket, Racket),
    scores: (i32, i32),
    ball: Ball,
    state: PongState,
    point_sound: Sound,
    racket_sound: Sound,
    wall_sound: Sound,
}

impl Pong {
    async fn new() -> Self {
        Self {
            rackets: (Racket::new(Side::Left), Racket::new(Side::Right)),
            ball: Ball::new(None),
            scores: (0, 0),
            state: PongState::Playing,
            point_sound: load_sound_from_bytes(include_bytes!("../sounds/point.wav"))
                .await
                .expect("load point sound file"),
            racket_sound: load_sound_from_bytes(include_bytes!("../sounds/racket.wav"))
                .await
                .expect("load racket sound file"),
            wall_sound: load_sound_from_bytes(include_bytes!("../sounds/wall.wav"))
                .await
                .expect("load wall sound file"),
        }
    }

    fn reset(&mut self) {
        self.rackets = (Racket::new(Side::Left), Racket::new(Side::Right));
        self.ball = Ball::new(None);
        self.scores = (0, 0);
        self.state = PongState::Playing;
    }

    fn update_racket_collisions(&mut self) {
        for racket in [&mut self.rackets.0, &mut self.rackets.1] {
            racket.pos.1 = racket.pos.1.clamp(0., WINDOW_HEIGHT - RACKET_SIZE.1);
        }
    }

    fn update_ball_collisions(&mut self) {
        const DX: f32 = 0.1;

        if self.ball.pos.0 < 0. {
            self.state = PongState::Point(Side::Right);
            return;
        }

        if self.ball.pos.0 + BALL_SIZE > WINDOW_WIDTH {
            self.state = PongState::Point(Side::Left);
            return;
        }

        if self.ball.pos.1 < 0. {
            self.ball.pos.1 = 0.;
            self.ball.dir.1 = self.ball.dir.1.abs();
            self.state = PongState::WallBounce;
            return;
        }

        if self.ball.pos.1 + BALL_SIZE > WINDOW_HEIGHT {
            self.ball.pos.1 = WINDOW_HEIGHT - BALL_SIZE;
            self.ball.dir.1 = -self.ball.dir.1.abs();
            self.state = PongState::WallBounce;
            return;
        }

        let ball_rect = Rect::new(self.ball.pos.0, self.ball.pos.1, BALL_SIZE, BALL_SIZE);
        for racket in [&self.rackets.0, &self.rackets.1] {
            let racket_rect = match racket.side {
                Side::Left => {
                    if self.ball.dir.0 > 0. {
                        continue;
                    }
                    Rect::new(
                        racket.pos.0 + RACKET_SIZE.0 - DX,
                        racket.pos.1,
                        DX * 2.,
                        RACKET_SIZE.1,
                    )
                }
                Side::Right => {
                    if self.ball.dir.0 < 0. {
                        continue;
                    }
                    Rect::new(racket.pos.0 - DX, racket.pos.1, DX * 2., RACKET_SIZE.1)
                }
            };

            let Some(rect) = racket_rect.intersect(ball_rect) else {
                continue;
            };

            self.ball.dir.0 = match racket.side {
                Side::Left => self.ball.dir.0.abs(),
                Side::Right => -self.ball.dir.0.abs(),
            };
            self.ball.dir.1 = (rect.center().y - racket_rect.center().y) / (racket_rect.h * 0.5);
            self.state = PongState::RacketBounce;
        }
    }

    fn update_score(&mut self, point_side: Side) {
        let score = match point_side {
            Side::Left => &mut self.scores.0,
            Side::Right => &mut self.scores.1,
        };

        *score += 1;
        self.state = if *score >= WIN_SCORE {
            PongState::Winner(point_side, get_time())
        } else {
            PongState::NewRound(point_side.toggle())
        };
    }

    fn update(&mut self) {
        let inputs = self.read_inputs();

        if inputs.contains(&Input::Quit) {
            self.state = PongState::Exit
        }

        match self.state {
            PongState::NewRound(side) => {
                self.ball = Ball::new(Some(side));
                self.state = PongState::Playing;
            }
            PongState::Playing => {
                if inputs.contains(&Input::Up(Side::Left)) {
                    self.rackets.0.slide(-RACKET_SPEED);
                }
                if inputs.contains(&Input::Down(Side::Left)) {
                    self.rackets.0.slide(RACKET_SPEED);
                }
                if inputs.contains(&Input::Up(Side::Right)) {
                    self.rackets.1.slide(-RACKET_SPEED);
                }
                if inputs.contains(&Input::Down(Side::Right)) {
                    self.rackets.1.slide(RACKET_SPEED);
                }
                self.update_racket_collisions();
                self.ball.fly();
                self.update_ball_collisions();
            }
            PongState::WallBounce | PongState::RacketBounce => {
                self.state = PongState::Playing;
            }
            PongState::Point(side) => {
                self.update_score(side);
            }
            PongState::Winner(_, at) => {
                if get_time() - at > WIN_SCREEN_SECS && !inputs.is_empty() {
                    self.reset();
                }
            }
            PongState::Exit => {}
        }
    }

    fn read_inputs(&mut self) -> Vec<Input> {
        let mut inputs = Vec::new();

        for key in get_keys_down() {
            match key {
                KeyCode::W => inputs.push(Input::Up(Side::Left)),
                KeyCode::S => inputs.push(Input::Down(Side::Left)),
                KeyCode::Up => inputs.push(Input::Up(Side::Right)),
                KeyCode::Down => inputs.push(Input::Down(Side::Right)),

                #[cfg(not(target_family = "wasm"))]
                KeyCode::Q => inputs.push(Input::Quit),

                _ => inputs.push(Input::Unknown),
            }
        }

        let scale_y = screen_height() / WINDOW_HEIGHT;
        for touch in touches() {
            let (side, racket_y) = if touch.position.x < screen_width() * 0.5 {
                (Side::Left, self.rackets.0.pos.1)
            } else {
                (Side::Right, self.rackets.1.pos.1)
            };
            if touch.position.y < (racket_y + RACKET_SIZE.1 * 0.25) * scale_y {
                inputs.push(Input::Up(side));
            } else if touch.position.y > (racket_y + RACKET_SIZE.1 * 0.75) * scale_y {
                inputs.push(Input::Down(side));
            }
        }

        inputs
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
            "(Press any key to play again)",
            40.,
            WINDOW_HEIGHT * 0.5 + 100.,
        );
    }

    fn draw(&self) {
        match self.state {
            PongState::Winner(side, _) => self.draw_winner(side),
            _ => {
                self.draw_scores();
                self.rackets.0.draw();
                self.rackets.1.draw();
                self.ball.draw();
            }
        }
    }

    fn play_sounds(&self) {
        match self.state {
            PongState::WallBounce => play_sound_once(&self.wall_sound),
            PongState::RacketBounce => play_sound_once(&self.racket_sound),
            PongState::Point(_) => play_sound_once(&self.point_sound),
            _ => {}
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

    let mut pong = Pong::new().await;

    loop {
        set_camera(&render_camera);

        clear_background(BACKGROUND_COLOR);

        pong.update();
        if matches!(pong.state(), PongState::Exit) {
            break;
        }
        pong.draw();
        pong.play_sounds();

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
// This shader is based on https://www.shadertoy.com/view/XtlSD7

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
