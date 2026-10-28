use std::collections::HashMap;
use std::time::Duration;

use mai::sim::{Character, Clip, World};

const WORLD: World = World {
    ground_y: 1040,
    left: 0,
    right: 1920,
};

fn idle_at(fps: u32, frames: u32) -> Character {
    let clips = HashMap::from([(
        "idle".to_string(),
        Clip {
            frames,
            fps,
            looping: true,
        },
    )]);
    Character::new(clips, &WORLD)
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn no_random() -> u32 {
    unreachable!("idle needs no randomness yet")
}

#[test]
fn starts_idle_on_frame_zero_centred_on_the_ground() {
    let mut mai = idle_at(10, 4);
    let draw = mai.step(&[], ms(0), &WORLD, &mut no_random);

    assert_eq!(draw.animation, "idle");
    assert_eq!(draw.frame, 0);
    assert_eq!((draw.x, draw.y), (960, 1040));
}

#[test]
fn idle_frame_advances_at_configured_fps_and_wraps() {
    // 10 fps → one frame per 100ms; 4 frames loop.
    let mut mai = idle_at(10, 4);
    let mut frames = vec![];
    for _ in 0..10 {
        frames.push(mai.step(&[], ms(50), &WORLD, &mut no_random).frame);
    }
    assert_eq!(frames, [0, 1, 1, 2, 2, 3, 3, 0, 0, 1]);
}

#[test]
fn many_small_steps_do_not_drift() {
    // 3 fps → frame period is 333.33..ms, which a per-step float accumulator rounds badly.
    let mut mai = idle_at(3, 1000);
    let mut draw = mai.step(&[], ms(0), &WORLD, &mut no_random);
    for _ in 0..10_000 {
        draw = mai.step(&[], ms(1), &WORLD, &mut no_random);
    }
    assert_eq!(draw.frame, 30); // 10s * 3fps
}

#[test]
fn non_looping_animation_stops_on_its_last_frame() {
    let clips = HashMap::from([(
        "idle".to_string(),
        Clip {
            frames: 3,
            fps: 10,
            looping: false,
        },
    )]);
    let mut mai = Character::new(clips, &WORLD);

    assert_eq!(mai.step(&[], ms(1000), &WORLD, &mut no_random).frame, 2);
}
