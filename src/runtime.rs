use super::Game;
use std::sync::Mutex;

pub(super) const ID: &str = "gamma_tweaks";
pub(super) const VERSION: &str = "0.2.0";
pub(super) const DESCRIPTION: &str = "Gamma slider, fullbright, sunlight and a light-level overlay.";
/// Vanilla's Brightness is 0..1; the override reaches past Bright.
pub(super) const MAX_GAMMA: f32 = 3.0;
/// Overlay radii the settings cycle, in blocks.
pub(super) const RADII: [i32; 4] = [8, 16, 24, 32];
/// Most overlay numbers drawn a frame (each is up to 14 short line segments).
pub(super) const MAX_MARKERS: usize = 128;
/// Blocks below and above the player's feet to scan for spawnable spaces.
const BELOW: i32 = 5;
const ABOVE: i32 = 9;

/// The mod's state. The registered settings page's rows get no `Game`, so the
/// state lives here: `sync` loads it from Horizon's per-mod settings once,
/// applies it to the game every tick and writes back what the page changed.
struct State {
    loaded: bool,
    /// The gamma override; `None` follows Video Settings' Brightness.
    gamma: Option<f32>,
    /// The Brightness option's value while the override is off (the slider's
    /// value then), refreshed each tick.
    video_gamma: f32,
    fullbright: bool,
    sunlight: bool,
    overlay: bool,
    /// Only mark spaces whose light is 0, where mobs can spawn.
    spawnable_only: bool,
    /// Index into `RADII`.
    radius: usize,
    /// A setter ran; `sync` writes the settings and clears it.
    dirty: bool,
}

static STATE: Mutex<State> = Mutex::new(State {
    loaded: false,
    gamma: None,
    video_gamma: 1.0,
    fullbright: false,
    sunlight: true,
    overlay: false,
    spawnable_only: false,
    radius: 1,
    dirty: false,
});

fn read<T>(f: impl FnOnce(&State) -> T) -> T {
    f(&STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

fn update(f: impl FnOnce(&mut State)) {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()));
}

fn write(f: impl FnOnce(&mut State)) {
    let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut s);
    s.dirty = true;
}

/// The override, `None` while it follows the Brightness option.
pub(super) fn gamma() -> Option<f32> {
    read(|s| s.gamma)
}
/// The gamma the lightmap gets: the override or the Brightness option.
pub(super) fn effective_gamma() -> f32 {
    read(|s| s.gamma.unwrap_or(s.video_gamma))
}
pub(super) fn set_gamma(value: Option<f32>) {
    write(|s| s.gamma = value.map(|v| v.clamp(0.0, MAX_GAMMA)));
}
pub(super) fn fullbright() -> bool {
    read(|s| s.fullbright)
}
pub(super) fn set_fullbright(on: bool) {
    write(|s| s.fullbright = on);
}
pub(super) fn sunlight() -> bool {
    read(|s| s.sunlight)
}
pub(super) fn set_sunlight(on: bool) {
    write(|s| s.sunlight = on);
}
pub(super) fn overlay() -> bool {
    read(|s| s.overlay)
}
pub(super) fn set_overlay(on: bool) {
    write(|s| s.overlay = on);
}
pub(super) fn spawnable_only() -> bool {
    read(|s| s.spawnable_only)
}
pub(super) fn set_spawnable_only(on: bool) {
    write(|s| s.spawnable_only = on);
}
pub(super) fn radius() -> i32 {
    read(|s| RADII[s.radius])
}
pub(super) fn cycle_radius(back: bool) {
    write(|s| {
        s.radius = if back {
            (s.radius + RADII.len() - 1) % RADII.len()
        } else {
            (s.radius + 1) % RADII.len()
        };
    });
}
/// Everything a settings page draws: `(override, effective, fullbright,
/// sunlight, overlay, spawnable_only, radius)`.
pub(super) fn page_state() -> (Option<f32>, f32, bool, bool, bool, bool, i32) {
    read(|s| {
        (
            s.gamma,
            s.gamma.unwrap_or(s.video_gamma),
            s.fullbright,
            s.sunlight,
            s.overlay,
            s.spawnable_only,
            RADII[s.radius],
        )
    })
}

/// `true` while the string is a registered toggle's stored form.
fn toggle(v: Option<String>) -> Option<bool> {
    match v.as_deref() {
        Some("true") => Some(true),
        Some("false") => Some(false),
        // Older files wrote 1/0.
        Some("1") => Some(true),
        Some("0") => Some(false),
        _ => None,
    }
}

fn load(game: &Game<'_>, s: &mut State) {
    // An empty gamma file means "follow the video setting".
    s.gamma = game
        .setting(ID, "gamma")
        .filter(|v| !v.is_empty())
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, MAX_GAMMA));
    s.fullbright = toggle(game.setting(ID, "fullbright"))
        .unwrap_or_else(|| game.settings.extras.fullbright);
    s.sunlight = toggle(game.setting(ID, "sunlight")).unwrap_or(true);
    s.overlay = toggle(game.setting(ID, "overlay")).unwrap_or(false);
    s.spawnable_only = toggle(game.setting(ID, "spawnable_only")).unwrap_or(false);
    s.radius = game
        .setting(ID, "radius")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|i| *i < RADII.len())
        .unwrap_or(1);
    s.video_gamma = game.settings.gamma;
    s.loaded = true;
}

/// Once a client tick, while a world is playing: load once, keep `settings`
/// and the renderer on our values and write back anything the page changed.
pub(super) fn sync(game: &mut Game<'_>) {
    {
        let mut s = STATE.lock().unwrap_or_else(|e| e.into_inner());
        if !s.loaded {
            load(game, &mut s);
        }
    }
    let (override_gamma, fullbright) = read(|s| (s.gamma, s.fullbright));
    if let Some(v) = override_gamma {
        // The override owns the lightmap's gamma — `apply_settings` would
        // otherwise restore the Brightness option's.
        game.settings.gamma = v;
        if let Some(r) = game.renderer.as_mut() {
            r.gamma = v;
        }
    } else {
        // The Brightness option moves freely; track it for the slider.
        update(|s| s.video_gamma = game.settings.gamma);
    }
    game.settings.extras.fullbright = fullbright;
    game.hold_fullbright();
    persist(game);
}

/// Scan the spaces above solid blocks near the player for the light-level
/// overlay. Levels are drawn as seven-segment digits by the mod crate.
pub(super) fn overlay_markers(game: &Game<'_>) -> Vec<::gamma_tweaks::overlay::Marker> {
    if !overlay() {
        return Vec::new();
    }
    let Some(player) = game.player.as_ref() else {
        return Vec::new();
    };
    const DARK: u32 = 0xFF_FF_55_55;
    const DIM: u32 = 0xFF_FF_BB_33;
    const LIT: u32 = 0xFF_55_FF_55;

    let facts = game.res.facts();
    let (px, py, pz) = (
        player.position.x.floor() as i32,
        player.position.y.floor() as i32,
        player.position.z.floor() as i32,
    );
    let r = radius();
    let ymin = (py - BELOW).max(game.world.shape.min_y + 1);
    let ymax = (py + ABOVE).min(game.world.shape.max_y() - 1);
    let spawnable_only = spawnable_only();
    let mut out = Vec::new();
    'done: for dx in -r..=r {
        for dz in -r..=r {
            let (x, z) = (px + dx, pz + dz);
            for y in ymin..=ymax {
                if facts.get(game.world.block(x, y, z)).full_collision
                    || !facts.get(game.world.block(x, y - 1, z)).full_collision
                {
                    continue;
                }
                let packed = i32::from(game.world.light(x, y, z));
                let level = (packed & 15).max((packed >> 4) & 15);
                if spawnable_only && level > 0 {
                    continue;
                }
                let color = if level == 0 {
                    DARK
                } else if level < 8 {
                    DIM
                } else {
                    LIT
                };
                out.push(::gamma_tweaks::overlay::Marker {
                    at: [f64::from(x) + 0.5, f64::from(y) + 0.02, f64::from(z) + 0.5],
                    level,
                    color,
                });
                if out.len() >= MAX_MARKERS {
                    break 'done;
                }
            }
        }
    }
    out
}

fn persist(game: &Game<'_>) {
    let snap = read(|s| {
        s.dirty.then_some((
            s.gamma,
            s.fullbright,
            s.sunlight,
            s.overlay,
            s.spawnable_only,
            s.radius,
        ))
    });
    let Some((gamma, fullbright, sunlight, overlay, spawnable_only, radius)) = snap else {
        return;
    };
    game.set_setting(
        ID,
        "gamma",
        &gamma.map_or_else(String::new, |v| v.to_string()),
    );
    game.set_setting(ID, "fullbright", if fullbright { "true" } else { "false" });
    game.set_setting(ID, "sunlight", if sunlight { "true" } else { "false" });
    game.set_setting(ID, "overlay", if overlay { "true" } else { "false" });
    game.set_setting(
        ID,
        "spawnable_only",
        if spawnable_only { "true" } else { "false" },
    );
    game.set_setting(ID, "radius", &radius.to_string());
    if let Some(dir) = Game::options_path().parent().map(|p| p.to_path_buf()) {
        game.settings.extras.save(&dir);
    }
    update(|s| s.dirty = false);
}
