//! The light-level overlay.
//!
//! The old mod drew numbers as world text during `native_world_submit`, which
//! the new game no longer has. Its world-geometry hook draws unlit lines, so
//! each level is now shown as two seven-segment digits lying on the block top.
//! The game-side module scans the world; this crate only holds the frame's
//! markers and draws them when the level is rendered.

use std::sync::Mutex;

/// One number above a solid block: world position, level 0..15 and ARGB colour.
#[derive(Clone, Copy)]
pub struct Marker {
    pub at: [f64; 3],
    pub level: i32,
    pub color: u32,
}

static MARKERS: Mutex<Vec<Marker>> = Mutex::new(Vec::new());

/// Replace the frame's markers (called from the game-side tick module).
pub fn set(markers: Vec<Marker>) {
    if let Ok(mut m) = MARKERS.lock() {
        *m = markers;
    }
}

/// A snapshot for drawing, taken once per view.
pub fn markers() -> Vec<Marker> {
    MARKERS
        .lock()
        .map_or_else(|e| e.into_inner().clone(), |m| m.clone())
}

/// Which of the seven segments a digit lights, top `a` through middle `g`.
fn segments(digit: i32) -> u8 {
    match digit {
        0 => 0b0111_111,
        1 => 0b0000_110,
        2 => 0b1011_011,
        3 => 0b1001_111,
        4 => 0b1100_110,
        5 => 0b1101_101,
        6 => 0b1101_111,
        7 => 0b0001_110,
        8 => 0b1111_111,
        9 => 0b1101_111,
        _ => 0,
    }
}

/// Add one digit's lines, with its top-left at `at` and its size in blocks.
fn digit(lines: &mut Vec<mod_api::WorldLine>, at: [f64; 3], digit: i32, color: u32) {
    const W: f64 = 0.26;
    const H: f64 = 0.42;
    const WIDTH: f32 = 1.5;
    let bits = segments(digit);
    let pos = |u: f64, v: f64| [at[0] + u, at[1], at[2] + v];
    // (u0, v0, u1, v1) for a, b, c, d, e, f, g.
    let seg = [
        (0.0, 0.0, W, 0.0),
        (W, 0.0, W, H / 2.0),
        (W, H / 2.0, W, H),
        (0.0, H, W, H),
        (0.0, H / 2.0, 0.0, H),
        (0.0, 0.0, 0.0, H / 2.0),
        (0.0, H / 2.0, W, H / 2.0),
    ];
    for (i, (u0, v0, u1, v1)) in seg.into_iter().enumerate() {
        if bits & (1 << i) != 0 {
            lines.push(mod_api::WorldLine::new(
                pos(u0, v0),
                pos(u1, v1),
                color,
                WIDTH,
                mod_api::WorldDepth::Tested,
            ));
        }
    }
}

/// Turn the scanned levels into world-space geometry for one view.
pub fn lines(markers: &[Marker]) -> Vec<mod_api::WorldLine> {
    let mut lines = Vec::with_capacity(markers.len() * 14);
    for m in markers {
        let level = m.level.clamp(0, 15);
        if level < 10 {
            digit(&mut lines, [m.at[0] - 0.13, m.at[1], m.at[2] - 0.21], level, m.color);
        } else {
            digit(&mut lines, [m.at[0] - 0.30, m.at[1], m.at[2] - 0.21], level / 10, m.color);
            digit(&mut lines, [m.at[0] + 0.04, m.at[1], m.at[2] - 0.21], level % 10, m.color);
        }
    }
    lines
}
