/// Timing of one Animation, as the simulation sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub frames: u32,
    pub fps: u32,
    pub looping: bool,
}

use std::collections::HashMap;
use std::time::Duration;

/// Animations the Character picks between on its own, with their relative weights.
pub const ANIMATION_WEIGHTS: &[(&str, u32)] = &[("idle", 3), ("walk", 2), ("sit", 1)];
/// How long one chosen Animation is held: `MIN_HOLD` plus up to `HOLD_SPREAD`.
const MIN_HOLD: Duration = Duration::from_secs(3);
const HOLD_SPREAD: Duration = Duration::from_secs(5);
/// Walking speed in screen pixels per second.
const WALK_SPEED: f64 = 60.0;
/// How often position is updated while walking, independent of the sprite's fps.
const MOVE_HZ: u32 = 30;
/// How far the mouse may move between press and release and still count as a click, like
/// Windows' default drag threshold.
const CLICK_SLOP: i32 = 4;
/// Downward acceleration of a dropped Character, in screen pixels per second squared.
const GRAVITY: f64 = 2500.0;

/// Where the Character may be: the Ground line and the range its bottom-centre may take, in
/// screen pixels.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub ground_y: i32,
    pub left: i32,
    pub right: i32,
}

/// Mouse input since the last step, in screen pixels. Only input over the Character's visible pixels
/// arrives, except while the button is held down after a press on them.
#[derive(Clone, Copy, Debug)]
pub enum MouseEvent {
    Press { x: i32, y: i32 },
    Move { x: i32, y: i32 },
    Release { x: i32, y: i32 },
    RightClick { x: i32, y: i32 },
}

/// What the shell should show: one frame of one Animation, anchored at its bottom-centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Draw {
    pub animation: &'static str,
    pub frame: u32,
    pub x: i32,
    pub y: i32,
    /// Draw the frame flipped left-to-right; sprites face right.
    pub mirrored: bool,
}

pub struct Character {
    clips: HashMap<String, Clip>,
    animation: &'static str,
    /// Time since the current Animation started; frames derive from it so steps never drift.
    played: Duration,
    /// Time left before the next Animation is chosen.
    until_next_choice: Duration,
    facing_left: bool,
    x: f64,
    y: f64,
    /// Where the button went down, until the mouse moves too far for a click.
    press: Option<(i32, i32)>,
    /// While dragged: the offset from the cursor to the bottom-centre.
    grab: (f64, f64),
    /// While falling: downward speed in pixels per second.
    fall_speed: f64,
    /// While reacting: the Animation to go back to and how far it had played.
    resume: Option<(&'static str, Duration)>,
}

impl Character {
    /// `clips` must hold every Animation in `assets::REQUIRED`.
    pub fn new(clips: HashMap<String, Clip>, world: &World) -> Self {
        let x = world.left + (world.right - world.left) / 2;
        Self {
            clips,
            animation: "idle",
            played: Duration::ZERO,
            until_next_choice: Duration::ZERO,
            facing_left: false,
            x: x as f64,
            y: world.ground_y as f64,
            press: None,
            grab: (0.0, 0.0),
            fall_speed: 0.0,
            resume: None,
        }
    }

    /// Advances the simulation by `elapsed` and returns what to draw.
    pub fn step(
        &mut self,
        events: &[MouseEvent],
        elapsed: Duration,
        world: &World,
        random: &mut dyn FnMut() -> u32,
    ) -> Draw {
        self.played += elapsed;
        match self.animation {
            "fall" => self.fall(elapsed, world),
            "land" if self.played_through() => {
                (self.animation, self.played) = ("idle", Duration::ZERO);
                self.until_next_choice = Duration::ZERO;
            }
            "react" if self.played_through() => {
                let (animation, played) = self.resume.take().unwrap();
                (self.animation, self.played) = (animation, played);
            }
            _ => {}
        }
        // Reacting, being dragged, falling and landing pause walking and the next choice.
        if !self.held() && self.animation != "react" {
            if self.walking() {
                self.walk(elapsed, world);
            }
            self.until_next_choice = self.until_next_choice.saturating_sub(elapsed);
            if self.until_next_choice.is_zero() {
                self.choose(random);
            }
        }
        for &event in events {
            self.mouse(event);
        }

        let clip = self.clips[self.animation];
        let shown = (self.played.as_nanos() * clip.fps as u128 / 1_000_000_000) as u64;
        let frame = if clip.looping {
            shown % clip.frames as u64
        } else {
            shown.min(clip.frames as u64 - 1)
        };
        Draw {
            animation: self.animation,
            frame: frame as u32,
            x: self.x.round() as i32,
            y: self.y.round() as i32,
            mirrored: self.walking() && self.facing_left,
        }
    }

    /// How long the shell may wait before the next step changes what is drawn.
    pub fn next_step_in(&self) -> Duration {
        let fps = if self.walking() || self.animation == "fall" {
            MOVE_HZ
        } else {
            self.clips[self.animation].fps
        };
        Duration::from_secs(1) / fps
    }

    fn mouse(&mut self, event: MouseEvent) {
        let near = |p: Option<(i32, i32)>, x: i32, y: i32| {
            p.is_some_and(|(px, py)| (x - px).abs() <= CLICK_SLOP && (y - py).abs() <= CLICK_SLOP)
        };
        match event {
            MouseEvent::Press { x, y } if !self.held() => self.press = Some((x, y)),
            MouseEvent::Move { x, y } => {
                if let Some((px, py)) = self.press
                    && !near(self.press, x, y)
                {
                    self.press = None;
                    self.resume = None;
                    self.grab = (self.x - px as f64, self.y - py as f64);
                    (self.animation, self.played) = ("dragged", Duration::ZERO);
                }
                if self.animation == "dragged" {
                    self.x = x as f64 + self.grab.0;
                    self.y = y as f64 + self.grab.1;
                }
            }
            MouseEvent::Release { x, y } => {
                let pressed = self.press.take();
                if self.animation == "dragged" {
                    (self.animation, self.played) = ("fall", Duration::ZERO);
                    self.fall_speed = 0.0;
                } else if near(pressed, x, y) {
                    self.react();
                }
            }
            _ => {}
        }
    }

    /// Drops toward the Ground; on contact, lands there pulled fully on screen.
    fn fall(&mut self, elapsed: Duration, world: &World) {
        let t = elapsed.as_secs_f64();
        self.fall_speed += GRAVITY * t;
        self.y += self.fall_speed * t;
        if self.y >= world.ground_y as f64 {
            self.y = world.ground_y as f64;
            self.x = self.x.clamp(world.left as f64, world.right as f64);
            (self.animation, self.played) = ("land", Duration::ZERO);
        }
    }

    /// The user has the Character: dragging, falling or landing.
    fn held(&self) -> bool {
        matches!(self.animation, "dragged" | "fall" | "land")
    }

    /// Whether the current Animation has shown all its frames once.
    fn played_through(&self) -> bool {
        let clip = self.clips[self.animation];
        self.played >= Duration::from_secs(clip.frames as u64) / clip.fps
    }

    /// Plays `react` from its first frame; a click while reacting restarts it.
    fn react(&mut self) {
        self.resume.get_or_insert((self.animation, self.played));
        self.animation = "react";
        self.played = Duration::ZERO;
    }

    fn walking(&self) -> bool {
        self.animation == "walk"
    }

    fn walk(&mut self, elapsed: Duration, world: &World) {
        let distance = WALK_SPEED * elapsed.as_secs_f64();
        self.x += if self.facing_left {
            -distance
        } else {
            distance
        };
        if self.x >= world.right as f64 {
            self.x = world.right as f64;
            self.facing_left = true;
        } else if self.x <= world.left as f64 {
            self.x = world.left as f64;
            self.facing_left = false;
        }
    }

    /// Picks the next Animation by weight; picking the current one again keeps its frames running.
    fn choose(&mut self, random: &mut dyn FnMut() -> u32) {
        let total: u32 = ANIMATION_WEIGHTS.iter().map(|w| w.1).sum();
        let mut roll = random() % total;
        let mut name = ANIMATION_WEIGHTS[0].0;
        for &(candidate, weight) in ANIMATION_WEIGHTS {
            if roll < weight {
                name = candidate;
                break;
            }
            roll -= weight;
        }
        let spread = HOLD_SPREAD.as_millis() as u32;
        self.until_next_choice = MIN_HOLD + Duration::from_millis((random() % spread) as u64);
        if name != self.animation {
            self.animation = name;
            self.played = Duration::ZERO;
        }
        if self.walking() {
            self.facing_left = random() % 2 == 1;
        }
    }
}
