extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use std::collections::LinkedList;

use std::iter::FromIterator;

use glutin_window::GlutinWindow as Window;
use graphics::types::{Color, Rectangle};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, ButtonEvent, EventLoop, Key};
use rand::distributions::{IndependentSample, Range};

#[derive(Clone, PartialEq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}
struct Food {
    position_x: i32,
    position_y: i32,
    color: Color,
}

impl Food {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        let square = graphics::rectangle::square(
            (self.position_x * 20) as f64,
            (self.position_y * 20) as f64,
            20f64,
        );

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(self.color, square, transform, gl)
        });
    }
}
struct Snake {
    body: LinkedList<(i32, i32)>,
    direction: Direction,
    color: Color,
}

impl Snake {
    fn grow(&mut self) {
        if let Some(&(tail_x, tail_y)) = self.body.back() {
            self.body.push_back((tail_x, tail_y - 1));
        }
    }

    fn collide(&self, food: &Food) -> bool {
        match self.body.front() {
            Some(&snake_head) => {
                return snake_head.0 == food.position_x && snake_head.1 == food.position_y
            }
            None => return false,
        };
    }
    fn make_snake_squares(&self) -> Vec<Rectangle> {
        self.body
            .iter()
            .map(|(x, y)| graphics::rectangle::square((x * 20) as f64, (y * 20) as f64, 20f64))
            .collect()
    }

    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        gl.draw(args.viewport(), |c, gl| {
            self.make_snake_squares().into_iter().for_each(|square| {
                let transform = c.transform;
                graphics::rectangle(self.color, square, transform, gl)
            });
        });
    }

    fn update(&mut self) {
        let mut new_head = (self.body.front().expect("No body.")).clone();

        self.move_head_in_direction(&mut new_head);

        self.render_new_positon(new_head);
    }

    fn move_head_in_direction(&self, new_head: &mut (i32, i32)) {
        match self.direction {
            Direction::Down => new_head.1 += 1,
            Direction::Up => new_head.1 -= 1,
            Direction::Right => new_head.0 += 1,
            Direction::Left => new_head.0 -= 1,
        }
    }

    fn render_new_positon(&mut self, new_head: (i32, i32)) {
        self.body.push_front(new_head);
        self.body.pop_back().unwrap();
    }
}

const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
struct Game {
    gl: GlGraphics, // OpenGL drawing backend.
    snake: Snake,
    food: Food,
}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        self.set_background_color(GREEN, &args);
        self.snake.render(&mut self.gl, &args);
        self.food.render(&mut self.gl, &args)
    }

    fn update(&mut self) {
        
        self.snake.update();
        
        let between = Range::new(0, 30);
        let mut rng = rand::thread_rng();

        if self.snake.collide(&self.food) {
            self.snake.grow();
            self.food.position_x = between.ind_sample(&mut rng);
            self.food.position_y = between.ind_sample(&mut rng);
        }
    }

    fn pressed(&mut self, button_pressed: &Button) {
        if self.is_in_opposite_direction(button_pressed) {
            return;
        }
        self.change_direction(&button_pressed)
    }

    fn set_background_color(&mut self, color: [f32; 4], args: &RenderArgs) {
        self.gl.draw(args.viewport(), |_c, gl| {
            graphics::clear(color, gl);
        });
    }

    fn is_in_opposite_direction(&self, button_pressed: &Button) -> bool {
        match button_pressed {
            Button::Keyboard(Key::Up) => self.snake.direction == Direction::Down,
            Button::Keyboard(Key::Down) => self.snake.direction == Direction::Up,
            Button::Keyboard(Key::Left) => self.snake.direction == Direction::Right,
            Button::Keyboard(Key::Right) => self.snake.direction == Direction::Left,
            _ => false,
        }
    }
    fn change_direction(&mut self, button_pressed: &Button) {
        let last_direction = self.snake.direction.clone();
        self.snake.direction = match button_pressed {
            &Button::Keyboard(Key::Up) => Direction::Up,
            &Button::Keyboard(Key::Down) => Direction::Down,
            &Button::Keyboard(Key::Left) => Direction::Left,
            &Button::Keyboard(Key::Right) => Direction::Right,
            _ => last_direction,
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        snake: Snake {
            body: LinkedList::from_iter(vec![(0, 0), (0, 1)].into_iter()),
            direction: Direction::Right,
            color: RED,
        },
        food: Food {
            position_x: 0,
            position_y: 0,
            color: [0.0, 0.0, 0.0, 1.0],
        },
    };

    let mut events = Events::new(EventSettings::new()).ups(10);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            game.render(&args);
        }

        if let Some(_) = e.update_args() {
            game.update();
        }

        if let Some(args) = e.button_args() {
            game.pressed(&args.button)
        }
    }
}
