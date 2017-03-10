use glium::glutin::{Event, MouseButton, VirtualKeyCode, MouseScrollDelta, ElementState};
use glium::{Surface, DrawParameters};
use glium::backend::glutin_backend::GlutinFacade;
use glium;

use std::time::{Instant, Duration};
use std::collections::HashMap;

pub struct Project<'a> {
    // reference back to the gl context
    display: &'a GlutinFacade,
    // when was the program Sized
    start: Instant,
    // when was draw last called
    last_draw: Option<Instant>,
    // the display size
    screen_size: (u32, u32),
    // last mouse position (if on screen)
    mouse_pos: Option<(f32, f32)>,
    // are keys down
    keys: HashMap<VirtualKeyCode, bool>,
    // are mouse buttons down
    mouse: HashMap<MouseButton, bool>,
}

impl<'a> Project<'a> {
    pub fn new(display: &'a GlutinFacade, start_size: (u32, u32)) -> Project<'a> {
        Project { 
            display: display,
            start: Instant::now(),
            last_draw: None,
            screen_size: start_size,
            mouse_pos: None,
            keys: HashMap::new(),
            mouse: HashMap::new(),
        }
    }

    pub fn event(&mut self, event: &Event) {
        use Event::*;
        match *event {
            KeyboardInput(state, _, Some(code)) => {
                use self::ElementState::*;

                match state {
                    Pressed => self.keys.insert(code, true),
                    Released => self.keys.insert(code, false),
                };

                // TODO on-press events
            },
            Resized(w, h) => {
                self.screen_size = (w, h)

                // TODO resize FBOs
            },
            MouseMoved(x, y) => {
                let (w, h) = self.screen_size;
                if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                    self.mouse_pos = None;
                } else {
                    self.mouse_pos = Some((
                        x as f32 / w as f32 * 2.0 - 1.0,
                        y as f32 / h as f32 * 2.0 - 1.0
                    ))
                }
            }, 
            MouseLeft => self.mouse_pos = None,
            MouseInput(state, butt) => {
                use self::ElementState::*;

                match state {
                    Pressed => self.mouse.insert(butt, true),
                    Released => self.mouse.insert(butt, false),
                };

                // TODO
            }
            MouseWheel(MouseScrollDelta::LineDelta(_, y), _) => {
                // TODO
            },
            _ => (),
        }
    }

    pub fn key_down(&self, code: VirtualKeyCode) -> bool {
        self.keys.get(&code).cloned().unwrap_or(false)
    }

    pub fn mouse_down(&self, butt: MouseButton) -> bool {
        self.mouse.get(&butt).cloned().unwrap_or(false)
    }

    fn update_draw_timer(&mut self) -> (f64, f64) {
        let now = Instant::now();
        self.last_draw = Some(now);

        let elapsed = now - self.start;
        let delta = now - self.last_draw.unwrap_or(now);

        (duration_to_secs(&elapsed), duration_to_secs(&delta))
    }

    pub fn draw<S: Surface>(&mut self, draw: &mut S) {
        let (elapsed, delta) = self.update_draw_timer();

        draw.clear_color_and_depth((0.01, 0.01, 0.02, 1.0), 1.0);

        let params = DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        //=================//
        //                 //
        // TODO Draw stuff //
        //                 //
        //=================//
    }   
}

fn duration_to_secs(dur: &Duration) -> f64 {
    dur.as_secs() as f64 + dur.subsec_nanos() as f64 * 1e-9f64
}