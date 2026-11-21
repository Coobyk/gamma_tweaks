use super::umbx_gamma as gamma;
use super::Game;

/// While a world is playing: keep the statics on the game — the override owns
/// `settings.gamma` and the renderer's, our flag owns `extras.fullbright` — and
/// pick up what the settings page changed.
pub(super) fn tick(game: &mut Game<'_>) {
    if !game.native_window() || !game.playing() {
        ::gamma_tweaks::overlay::set(Vec::new());
        return;
    }
    gamma::sync(game);
    ::gamma_tweaks::overlay::set(gamma::overlay_markers(game));
}

/// `end_flash_sky`: the frame's sky light factor, before the End's lightning
/// flash adds to it. Zero is no sunlight: block light and the ambient floor
/// remain, the sky itself is untouched.
pub(super) fn sky(game: &Game<'_>, sky: &mut horizon_client::environment::Sky) {
    if game.native_window() && game.playing() && !gamma::sunlight() {
        sky.sky_light_factor = 0.0;
    }
}
