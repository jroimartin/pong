//! The classic table tennis–themed video game.
use std::fmt;

use macroquad::color::{self, colors};
use macroquad::input::{self, KeyCode};
use macroquad::math;
use macroquad::shapes;
use macroquad::text;
use macroquad::time;
use macroquad::window;

const BACKGROUND_COLOR: color::Color = colors::BLACK;
const FOREGROUND_COLOR: color::Color = colors::WHITE;

const RACKET_SIZE: (f32, f32) = (20., 100.);
const RACKET_MARGIN: f32 = 20.;
const RACKET_SPEED: f32 = 500.;

const BALL_SIZE: f32 = 20.;
const BALL_INIT_SPEED: f32 = 150.;
const BALL_ACCEL: f32 = 10.;

const WIN_SCORE: i32 = 2;

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
            Side::Right => window::screen_width() - RACKET_MARGIN - RACKET_SIZE.0,
        };
        let pos_y = window::screen_height() * 0.5 - RACKET_SIZE.1 * 0.5;
        Self {
            side,
            pos: (pos_x, pos_y),
        }
    }

    fn slide(&mut self, speed: f32) {
        let pos_y = self.pos.1 + speed * time::get_frame_time();
        self.pos.1 = if pos_y < 0. {
            0.
        } else if pos_y + RACKET_SIZE.1 > window::screen_height() {
            window::screen_height() - RACKET_SIZE.1
        } else {
            pos_y
        };
    }

    fn draw(&self) {
        shapes::draw_rectangle(
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
        let x = window::screen_width() * 0.5 - BALL_SIZE * 0.5;
        let y = window::screen_height() * 0.5 - BALL_SIZE * 0.5;
        let rnddir = || -> f32 { ((((time::get_time() * 1e6) as i32) & 1) * 2 - 1) as f32 };
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
        let ft = time::get_frame_time();

        let delta = self.speed * ft;

        self.pos.0 += self.dir.0 * delta;

        let y = self.pos.1 + self.dir.1 * delta;
        (self.pos.1, self.dir.1) = if y < 0. {
            (0., -self.dir.1)
        } else if y + BALL_SIZE > window::screen_height() {
            (window::screen_height() - BALL_SIZE, -self.dir.1)
        } else {
            (y, self.dir.1)
        };

        self.speed += ft * BALL_ACCEL;
    }

    fn draw(&self) {
        shapes::draw_rectangle(
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
    left_racket: Racket,
    right_racket: Racket,
    left_score: i32,
    right_score: i32,
    ball: Ball,
    state: PongState,
}

impl Pong {
    fn new() -> Self {
        Self {
            left_racket: Racket::new(Side::Left),
            right_racket: Racket::new(Side::Right),
            ball: Ball::new(None),
            left_score: 0,
            right_score: 0,
            state: PongState::Playing,
        }
    }

    fn reset(&mut self) {
        self.ball = Ball::new(None);
        self.left_score = 0;
        self.right_score = 0;
        self.state = PongState::Playing;
    }

    fn update_scores(&mut self) {
        if self.ball.pos.0 < 0. {
            self.right_score += 1;
            self.state = if self.right_score >= WIN_SCORE {
                PongState::Winner(Side::Right)
            } else {
                PongState::NewRound(Side::Left)
            };
            return;
        }
        if self.ball.pos.0 + BALL_SIZE > window::screen_width() {
            self.left_score += 1;
            self.state = if self.left_score >= WIN_SCORE {
                PongState::Winner(Side::Left)
            } else {
                PongState::NewRound(Side::Right)
            };
            return;
        }
    }

    fn update_collisions(&mut self) {
        for racket in [&self.left_racket, &self.right_racket] {
            let ball_rect = math::Rect::new(self.ball.pos.0, self.ball.pos.1, BALL_SIZE, BALL_SIZE);
            let racket_rect =
                math::Rect::new(racket.pos.0, racket.pos.1, RACKET_SIZE.0, RACKET_SIZE.1);

            let Some(rect) = racket_rect.intersect(ball_rect) else {
                continue;
            };

            match racket.side {
                Side::Left => self.ball.dir.0 = self.ball.dir.0.abs(),
                Side::Right => self.ball.dir.0 = -self.ball.dir.0.abs(),
            }

            let rect_cy = rect.y + rect.h * 0.5;
            let racket_rect_cy = racket_rect.y + racket_rect.h * 0.5;
            let ball_dir = (rect_cy - racket_rect_cy) / (racket_rect.h * 0.5);
            self.ball.dir.1 = ball_dir;
        }
    }

    fn update(&mut self) {
        if input::is_key_down(KeyCode::Q) {
            self.state = PongState::Exit
        }

        match self.state {
            PongState::NewRound(side) => {
                self.ball = Ball::new(Some(side));
                self.state = PongState::Playing;
            }
            PongState::Playing => {
                if input::is_key_down(KeyCode::W) {
                    self.left_racket.slide(-RACKET_SPEED);
                }
                if input::is_key_down(KeyCode::S) {
                    self.left_racket.slide(RACKET_SPEED);
                }

                if input::is_key_down(KeyCode::Up) {
                    self.right_racket.slide(-RACKET_SPEED);
                }
                if input::is_key_down(KeyCode::Down) {
                    self.right_racket.slide(RACKET_SPEED);
                }
                self.ball.update();
                self.update_collisions();
                self.update_scores();
            }
            PongState::Winner(_) => {
                if input::is_key_down(KeyCode::Space) {
                    self.reset();
                }
            }
            PongState::Exit => {}
        }
    }

    fn draw_scores(&self) {
        draw_text_center(
            &format!("{} - {}", self.left_score, self.right_score),
            75.0,
            30.0,
        );
    }

    fn draw_winner(&self, side: Side) {
        draw_text_center(
            &format!("{side} WON!"),
            150.0,
            window::screen_height() * 0.5,
        );
        draw_text_center(
            &format!("(Press SPACE to play again)"),
            40.,
            window::screen_height() * 0.5 + 100.,
        );
    }

    fn draw(&self) {
        match self.state {
            PongState::Playing | PongState::NewRound(_) => {
                self.draw_scores();
                self.left_racket.draw();
                self.right_racket.draw();
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
    let fps = format!("{:3} FPS", time::get_fps());
    text::draw_text(&fps, 10., 20., 20., colors::GREEN);
}

fn draw_text_center(text: &str, font_size: f32, y: f32) {
    let text_sz = text::measure_text(&text, None, font_size as u16, 1.);
    text::draw_text(
        &text,
        window::screen_width() * 0.5 - text_sz.width * 0.5,
        y - text_sz.height * 0.5 + text_sz.offset_y,
        font_size,
        FOREGROUND_COLOR,
    );
}

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: "PONG".to_owned(),
        window_width: 800,
        window_height: 600,
        ..window::Conf::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut pong = Pong::new();

    loop {
        window::clear_background(BACKGROUND_COLOR);

        pong.update();
        if matches!(pong.state(), PongState::Exit) {
            break;
        }
        pong.draw();

        #[cfg(debug_assertions)]
        draw_fps();

        window::next_frame().await;
    }
}
