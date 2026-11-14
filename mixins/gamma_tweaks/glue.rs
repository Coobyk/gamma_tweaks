use super::native_hooks::NativeMod;
use super::native_world_draw::NativeWorldDraw;
use super::umbx_gamma as gamma;
use super::Game;

fn on(game: &Game<'_>) -> bool {
    game.native_on(gamma::ID)
}

/// `native_frame`: register the Horizon Settings page once, then `sync` keeps
/// its statics on the game — the override owns `settings.gamma` and the
/// renderer's, our flag owns `extras.fullbright` — and persists what the
/// pages change.
pub(super) fn frame(game: &mut Game<'_>) {
    if !on(game) {
        return;
    }
    super::umbx_gamma_page::register();
    gamma::sync(game);
}

/// `end_flash_sky`: the frame's sky light factor, before the End's lightning
/// flash adds to it. Zero is no sunlight: block light and the ambient floor
/// remain, the sky itself is untouched.
pub(super) fn sky(game: &Game<'_>, sky: &mut horizon_client::environment::Sky) {
    if on(game) && !gamma::sunlight() {
        sky.sky_light_factor = 0.0;
    }
}

/// `native_world_submit`: the light-level overlay's numbers.
pub(super) fn world(game: &Game<'_>, draw: &mut NativeWorldDraw) {
    if on(game) {
        super::umbx_gamma_overlay::submit(game, draw);
    }
}

pub(super) fn listed(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: gamma::ID.into(),
        title: format!("Gamma Tweaks {}", gamma::VERSION),
        description: gamma::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}

fn onoff(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

fn labels() -> Vec<String> {
    let (_, effective, fullbright, sunlight, overlay, spawnable, radius) = gamma::page_state();
    vec![
        if gamma::gamma().is_some() {
            format!("Gamma: {}%", (effective * 100.0).round() as i32)
        } else {
            "Gamma: video setting".to_owned()
        },
        format!("Fullbright: {}", onoff(fullbright)),
        format!("Sunlight: {}", onoff(sunlight)),
        format!("Light-level overlay: {}", onoff(overlay)),
        format!("Only where mobs spawn: {}", onoff(spawnable)),
        format!("Overlay radius: {radius}"),
        "Gamma slider: Horizon Settings".to_owned(),
    ]
}

pub(super) fn page(game: &mut Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id != gamma::ID {
        return;
    }
    // Load the statics if no frame ran since the mod switched on.
    gamma::sync(game);
    buttons.extend(labels());
}

pub(super) fn click(game: &mut Game<'_>, mod_id: &str, button: usize) {
    if mod_id != gamma::ID {
        return;
    }
    gamma::sync(game);
    match button {
        0 => gamma::set_gamma(None),
        1 => gamma::set_fullbright(!gamma::fullbright()),
        2 => gamma::set_sunlight(!gamma::sunlight()),
        3 => gamma::set_overlay(!gamma::overlay()),
        4 => gamma::set_spawnable_only(!gamma::spawnable_only()),
        5 => gamma::cycle_radius(false),
        _ => {}
    }
}
