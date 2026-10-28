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

/// Where the Character may be: the Ground line and the range its bottom-centre may take, in
/// screen pixels.
#[derive(Clone, Copy, Debug)]
pub struct World {
    pub ground_y: i32,
    pub left: i32,
    pub right: i32,
}

/// Mouse input since the last step, in screen pixels. Not acted on yet.
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
    y: i32,
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
            y: world.ground_y,
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
        let _ = events;
        self.played += elapsed;
        if self.walking() {
            self.walk(elapsed, world);
        }
        self.until_next_choice = self.until_next_choice.saturating_sub(elapsed);
        if self.until_next_choice.is_zero() {
            self.choose(random);
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
            y: self.y,
            mirrored: self.walking() && self.facing_left,
        }
    }

    /// How long the shell may wait before the next step changes what is drawn.
    pub fn next_step_in(&self) -> Duration {
        let fps = if self.walking() {
            MOVE_HZ
        } else {
            self.clips[self.animation].fps
        };
        Duration::from_secs(1) / fps
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
