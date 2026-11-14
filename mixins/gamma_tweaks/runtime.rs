use super::Game;
use std::sync::Mutex;

pub(super) const ID: &str = "gamma_tweaks";
pub(super) const VERSION: &str = "0.2.0";
pub(super) const DESCRIPTION: &str = "Gamma slider, fullbright, sunlight and a light-level overlay.";
/// Vanilla's Brightness is 0..1; the override reaches past Bright.
pub(super) const MAX_GAMMA: f32 = 3.0;
/// Overlay radii the pages cycle, in blocks.
pub(super) const RADII: [i32; 4] = [8, 16, 24, 32];
/// Most overlay numbers drawn a frame.
pub(super) const MAX_MARKERS: usize = 512;

/// The mod's state. The registered settings page's `rows`/`act` get no `Game`,
/// so the state lives here: `sync` loads it from the files once, applies it to
/// the game every frame and writes back what either page changed.
struct State {
    loaded: bool,
    /// The gamma override; `None` follows Video Settings' Brightness.
    gamma: Option<f32>,
    /// The Brightness option's value while the override is off (the slider's
    /// value then), refreshed each frame.
    video_gamma: f32,
    fullbright: bool,
    sunlight: bool,
    overlay: bool,
    /// Only mark spaces whose light is 0, where mobs can spawn.
    spawnable_only: bool,
    /// Index into `RADII`.
    radius: usize,
    /// A setter ran; `sync` writes the files and clears it.
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
/// Everything a page draws: `(override, effective, fullbright, sunlight,
/// overlay, spawnable_only, radius)`.
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

fn load(game: &Game<'_>, s: &mut State) {
    s.gamma = game
        .native_setting(ID, "gamma")
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v.clamp(0.0, MAX_GAMMA));
    s.fullbright = match game.native_setting(ID, "fullbright").as_deref() {
        Some("1") => true,
        Some("0") => false,
        // First run: adopt the option's current value.
        _ => game.settings.extras.fullbright,
    };
    s.sunlight = game.native_setting(ID, "sunlight").as_deref() != Some("0");
    s.overlay = game.native_setting(ID, "overlay").as_deref() == Some("1");
    s.spawnable_only = game.native_setting(ID, "spawnable_only").as_deref() == Some("1");
    s.radius = game
        .native_setting(ID, "radius")
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|i| *i < RADII.len())
        .unwrap_or(1);
    s.video_gamma = game.settings.gamma;
    s.loaded = true;
}

/// `native_frame`, while the mod is on: load once, keep `settings` and the
/// renderer on our values and write back anything either page changed.
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
    game.native_set_setting(
        ID,
        "gamma",
        &gamma.map_or_else(String::new, |v| v.to_string()),
    );
    game.native_set_setting(ID, "fullbright", if fullbright { "1" } else { "0" });
    game.native_set_setting(ID, "sunlight", if sunlight { "1" } else { "0" });
    game.native_set_setting(ID, "overlay", if overlay { "1" } else { "0" });
    game.native_set_setting(ID, "spawnable_only", if spawnable_only { "1" } else { "0" });
    game.native_set_setting(ID, "radius", &radius.to_string());
    if let Some(dir) = Game::options_path().parent().map(|p| p.to_path_buf()) {
        game.settings.extras.save(&dir);
    }
    update(|s| s.dirty = false);
}
