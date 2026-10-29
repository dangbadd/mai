use std::collections::HashMap;
use std::time::Duration;

use mai::sim::{ANIMATION_WEIGHTS, Character, Clip, MouseEvent, World};

const WORLD: World = World {
    ground_y: 1040,
    left: 0,
    right: 1920,
};

fn clip(fps: u32, frames: u32) -> Clip {
    Clip {
        frames,
        fps,
        looping: true,
    }
}

/// Every required Animation, with `idle` timed as given.
fn character(idle: Clip, world: &World) -> Character {
    let clips = HashMap::from([
        ("idle".to_string(), idle),
        ("walk".to_string(), clip(8, 4)),
        ("sit".to_string(), clip(2, 2)),
        (
            "react".to_string(),
            Clip {
                frames: 5,
                fps: 10,
                looping: false,
            },
        ),
    ]);
    Character::new(clips, world)
}

fn idle_at(fps: u32, frames: u32) -> Character {
    character(clip(fps, frames), &WORLD)
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Always draws 0, which picks the first Animation, `idle`, for the shortest hold.
fn no_random() -> u32 {
    0
}

/// xorshift32: a tiny deterministic generator for seeded runs.
fn seeded(mut state: u32) -> impl FnMut() -> u32 {
    move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    }
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
    let mut mai = character(
        Clip {
            frames: 3,
            fps: 10,
            looping: false,
        },
        &WORLD,
    );

    assert_eq!(mai.step(&[], ms(1000), &WORLD, &mut no_random).frame, 2);
}

/// Which Animation is shown after each of `steps` 100ms steps.
fn sequence(seed: u32, steps: usize) -> Vec<&'static str> {
    let mut mai = idle_at(4, 4);
    let mut random = seeded(seed);
    (0..steps)
        .map(|_| mai.step(&[], ms(100), &WORLD, &mut random).animation)
        .collect()
}

#[test]
fn same_seed_gives_the_same_animation_sequence() {
    let a = sequence(7, 5_000);
    assert_eq!(a, sequence(7, 5_000));
    for name in ["idle", "walk", "sit"] {
        assert!(a.contains(&name), "{name} never chosen");
    }
    assert_ne!(a, sequence(8, 5_000));
}

#[test]
fn time_spent_in_each_animation_follows_the_weights() {
    // Every Animation draws its hold time from the same range, so time share tracks weight share.
    let shown = sequence(12345, 2_000_000); // ~200_000s, tens of thousands of choices
    let total: u32 = ANIMATION_WEIGHTS.iter().map(|w| w.1).sum();
    for &(name, weight) in ANIMATION_WEIGHTS {
        let share = shown.iter().filter(|&&a| a == name).count() as f64 / shown.len() as f64;
        let expected = weight as f64 / total as f64;
        assert!(
            (share - expected).abs() < 0.02,
            "{name}: {share} vs {expected}"
        );
    }
}

const NARROW: World = World {
    ground_y: 1040,
    left: 900,
    right: 1000,
};

/// Starts a long walk (rolls: walk, longest length, then the direction roll), then idles.
fn walking(direction_roll: u32) -> (Character, impl FnMut() -> u32) {
    let mut rolls = vec![3, 4999, direction_roll].into_iter();
    (character(clip(4, 4), &NARROW), move || {
        rolls.next().unwrap_or(0)
    })
}

/// Positions and mirroring over the first 2s of walking, stepped at 30Hz.
fn walk_path(direction_roll: u32) -> Vec<(i32, bool)> {
    let (mut mai, mut random) = walking(direction_roll);
    mai.step(&[], ms(0), &NARROW, &mut random);
    (0..60)
        .map(|_| {
            let d = mai.step(&[], ms(33), &NARROW, &mut random);
            assert_eq!(d.animation, "walk");
            (d.x, d.mirrored)
        })
        .collect()
}

#[test]
fn walking_into_the_right_edge_turns_around() {
    let path = walk_path(0);
    assert!(path.iter().all(|&(x, _)| (900..=1000).contains(&x)));
    assert!(!path[0].1, "faces right on the way out");
    let turn = path.iter().position(|&(_, m)| m).expect("turns around");
    assert_eq!(path[turn].0, 1000, "turns at the right edge");
    assert!(path.last().unwrap().0 < 1000, "walks back");
}

#[test]
fn walking_into_the_left_edge_turns_around() {
    let path = walk_path(1);
    assert!(path.iter().all(|&(x, _)| (900..=1000).contains(&x)));
    assert!(path[0].1, "faces left on the way out");
    let turn = path.iter().position(|&(_, m)| !m).expect("turns around");
    assert_eq!(path[turn].0, 900, "turns at the left edge");
    assert!(path.last().unwrap().0 > 900, "walks back");
}

#[test]
fn walking_steps_at_30hz_while_standing_steps_at_sprite_fps() {
    let mut mai = idle_at(4, 4);
    mai.step(&[], ms(0), &WORLD, &mut no_random);
    assert_eq!(mai.next_step_in(), ms(250));

    let (mut mai, mut random) = walking(0);
    mai.step(&[], ms(0), &NARROW, &mut random);
    assert_eq!(mai.next_step_in(), Duration::from_secs(1) / 30);
}

/// Press and release at `(x, y)`, then `moved_to` in between when given.
fn click(x: i32, y: i32, moved_to: Option<(i32, i32)>) -> Vec<MouseEvent> {
    let mut events = vec![MouseEvent::Press { x, y }];
    if let Some((x, y)) = moved_to {
        events.push(MouseEvent::Move { x, y });
    }
    events.push(MouseEvent::Release { x, y });
    events
}

#[test]
fn a_click_plays_react_once_then_the_previous_animation_resumes() {
    // react is 5 frames at 10fps: 500ms.
    let mut mai = idle_at(10, 4);
    mai.step(&[], ms(0), &WORLD, &mut no_random);
    assert_eq!(mai.step(&[], ms(200), &WORLD, &mut no_random).frame, 2);

    let draw = mai.step(&click(960, 1000, None), ms(0), &WORLD, &mut no_random);
    assert_eq!((draw.animation, draw.frame), ("react", 0));
    let draw = mai.step(&[], ms(450), &WORLD, &mut no_random);
    assert_eq!((draw.animation, draw.frame), ("react", 4));

    let draw = mai.step(&[], ms(100), &WORLD, &mut no_random);
    assert_eq!(draw.animation, "idle");
    assert_eq!(draw.frame, 2, "idle picks up where it left off");
}

#[test]
fn walking_resumes_in_place_after_react() {
    let (mut mai, mut random) = walking(0);
    mai.step(&[], ms(0), &NARROW, &mut random);
    let before = mai.step(&[], ms(100), &NARROW, &mut random);

    let draw = mai.step(&click(0, 0, None), ms(0), &NARROW, &mut random);
    assert_eq!(draw.animation, "react");
    let draw = mai.step(&[], ms(400), &NARROW, &mut random);
    assert_eq!(
        (draw.animation, draw.x),
        ("react", before.x),
        "stands still"
    );

    let draw = mai.step(&[], ms(200), &NARROW, &mut random);
    assert_eq!(draw.animation, "walk");
    assert!(draw.x > before.x, "walks on to the right");
}

#[test]
fn press_and_release_after_moving_away_is_not_a_click() {
    let mut mai = idle_at(10, 4);
    let draw = mai.step(
        &click(960, 1000, Some((990, 1000))),
        ms(0),
        &WORLD,
        &mut no_random,
    );
    assert_eq!(draw.animation, "idle");
}

#[test]
fn a_small_wobble_still_counts_as_a_click() {
    let mut mai = idle_at(10, 4);
    let draw = mai.step(
        &click(960, 1000, Some((962, 999))),
        ms(0),
        &WORLD,
        &mut no_random,
    );
    assert_eq!(draw.animation, "react");
}
