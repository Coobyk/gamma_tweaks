use super::Game;
use std::sync::Mutex;

pub(super) const ID: &str = "gamma_tweaks";
/// Vanilla's Brightness is 0..1; the override reaches past Bright.
pub(super) const MAX_GAMMA: f32 = 3.0;
/// Overlay radii the settings cycle, in blocks.
pub(super) const RADII: [i32; 4] = [8, 16, 24, 32];
/// Most overlay numbers drawn a frame (each is up to 14 short line segments).
pub(super) const MAX_MARKERS: usize = 128;
/// Blocks below and above the player's feet to scan for spawnable spaces.
const BELOW: i32 = 5;
const ABOVE: i32 = 9;

/// Defaults `STATE` starts from and `load` falls back on for a missing key, so
/// a fresh launch and an absent file agree. (`fullbright` is seeded from the
/// game's own extras flag instead, and `radius` indexes `RADII`.)
const DEFAULT_SUNLIGHT: bool = true;
const DEFAULT_OVERLAY: bool = false;
const DEFAULT_SPAWNABLE_ONLY: bool = false;
const DEFAULT_RADIUS: usize = 1;

/// The mod's state. The registered settings page's rows get no `Game`, so the
/// state lives here: `sync` loads it from Horizon's per-mod settings once,
/// applies it to the game every tick and re-reads what the page changed.
struct State {
    loaded: bool,
    /// The gamma override; `None` follows Video Settings' Brightness.
    gamma: Option<f32>,
    fullbright: bool,
    sunlight: bool,
    overlay: bool,
    /// Only mark spaces whose light is 0, where mobs can spawn.
    spawnable_only: bool,
    /// Index into `RADII`.
    radius: usize,
}

static STATE: Mutex<State> = Mutex::new(State {
    loaded: false,
    gamma: None,
    fullbright: false,
    sunlight: DEFAULT_SUNLIGHT,
    overlay: DEFAULT_OVERLAY,
    spawnable_only: DEFAULT_SPAWNABLE_ONLY,
    radius: DEFAULT_RADIUS,
});

fn read<T>(f: impl FnOnce(&State) -> T) -> T {
    f(&STATE.lock().unwrap_or_else(|e| e.into_inner()))
}

fn update(f: impl FnOnce(&mut State)) {
    f(&mut STATE.lock().unwrap_or_else(|e| e.into_inner()));
}

pub(super) fn sunlight() -> bool {
    read(|s| s.sunlight)
}
pub(super) fn overlay() -> bool {
    read(|s| s.overlay)
}
pub(super) fn spawnable_only() -> bool {
    read(|s| s.spawnable_only)
}
pub(super) fn radius() -> i32 {
    read(|s| RADII[s.radius])
}

/// `true` while the string is a registered toggle's stored form.
fn toggle(v: Option<String>) -> Option<bool> {
    match v.as_deref() {
        Some("true") => Some(true),
        Some("false") => Some(false),
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
    s.sunlight = toggle(game.setting(ID, "sunlight")).unwrap_or(DEFAULT_SUNLIGHT);
    s.overlay = toggle(game.setting(ID, "overlay")).unwrap_or(DEFAULT_OVERLAY);
    s.spawnable_only =
        toggle(game.setting(ID, "spawnable_only")).unwrap_or(DEFAULT_SPAWNABLE_ONLY);
    s.radius = game
        .setting(ID, "radius")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|i| *i < RADII.len())
        .unwrap_or(DEFAULT_RADIUS);
    s.loaded = true;
}

/// Re-read what the settings page wrote. Missing keys keep the current value.
fn refresh(game: &Game<'_>, s: &mut State) {
    if let Some(v) = game.setting(ID, "gamma") {
        s.gamma = if v.is_empty() {
            None
        } else {
            v.parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .map(|v| v.clamp(0.0, MAX_GAMMA))
        };
    }
    if let Some(v) = toggle(game.setting(ID, "fullbright")) {
        s.fullbright = v;
    }
    if let Some(v) = toggle(game.setting(ID, "sunlight")) {
        s.sunlight = v;
    }
    if let Some(v) = toggle(game.setting(ID, "overlay")) {
        s.overlay = v;
    }
    if let Some(v) = toggle(game.setting(ID, "spawnable_only")) {
        s.spawnable_only = v;
    }
    if let Some(i) = game
        .setting(ID, "radius")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|i| *i < RADII.len())
    {
        s.radius = i;
    }
}

/// Once a client tick, while a world is playing: load once and keep
/// `settings` and the renderer on our values.
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
    }
    game.settings.extras.fullbright = fullbright;
    game.hold_fullbright();
    // The settings page stores straight to the per-mod files; pull them back
    // every tick so a click takes effect without a restart.
    update(|s| refresh(game, s));
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
