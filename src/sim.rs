/// Timing of one Animation, as the simulation sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub frames: u32,
    pub fps: u32,
    pub looping: bool,
}

use std::collections::HashMap;
use std::time::Duration;

/// Where the Character may be: the Ground line and the screen edges, in screen pixels.
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
}

pub struct Character {
    clips: HashMap<String, Clip>,
    animation: &'static str,
    /// Time since the current Animation started; frames derive from it so steps never drift.
    played: Duration,
    x: i32,
    y: i32,
}

impl Character {
    /// `clips` must hold every Animation in `assets::REQUIRED`.
    pub fn new(clips: HashMap<String, Clip>, world: &World) -> Self {
        let x = world.left + (world.right - world.left) / 2;
        Self { clips, animation: "idle", played: Duration::ZERO, x, y: world.ground_y }
    }

    /// Advances the simulation by `elapsed` and returns what to draw.
    pub fn step(
        &mut self,
        events: &[MouseEvent],
        elapsed: Duration,
        world: &World,
        random: &mut dyn FnMut() -> u32,
    ) -> Draw {
        let _ = (events, world, random);
        self.played += elapsed;
        let clip = self.clips[self.animation];
        let shown = (self.played.as_nanos() * clip.fps as u128 / 1_000_000_000) as u64;
        let frame = if clip.looping {
            shown % clip.frames as u64
        } else {
            shown.min(clip.frames as u64 - 1)
        };
        Draw { animation: self.animation, frame: frame as u32, x: self.x, y: self.y }
    }
}
