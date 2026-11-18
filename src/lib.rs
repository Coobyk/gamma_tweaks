//! Gamma Tweaks' own crate. The client mixins under `src/mixins/` are
//! compiled into copies of the game functions they patch; the shared state
//! modules are injected by `[[native.module]]`.
//!
//! The settings page is Horizon's own: each option is registered here and
//! shows up under Options > Mods > Gamma Tweaks. The game-side modules read
//! and write the same keys through `Game::setting`/`Game::set_setting`.

pub mod overlay;

use mod_api::{
    ClientHost, Hooks, Registrar, RegistrationContext, RenderLevelStage, RenderLevelStageEvent,
    SettingSpec,
};

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// The light-level overlay, once the level has been drawn. The old mod's
    /// world-text submit hook is gone; the new draw context takes world lines,
    /// so each level is drawn as seven-segment digits on the block's top.
    fn render_level_stage(
        &self,
        _: &dyn ClientHost,
        event: &mut RenderLevelStageEvent<'_>,
    ) {
        if event.stage != RenderLevelStage::AfterLevel {
            return;
        }
        let markers = overlay::markers();
        let lines = overlay::lines(&markers);
        let _ = event.draw.lines(&lines);
    }

    fn register(&self, reg: &mut dyn Registrar, _: &RegistrationContext) {
        // Gamma is stored in the same 0..3 range the old slider used; no file
        // yet means "follow the video Brightness setting".
        let _ = reg.register_setting(SettingSpec::range(
            "gamma",
            "gamma_tweaks.setting.gamma",
            0.0,
            3.0,
            0.05,
            1.0,
        ));
        let _ = reg.register_setting(SettingSpec::toggle(
            "fullbright",
            "gamma_tweaks.setting.fullbright",
            false,
        ));
        let _ = reg.register_setting(SettingSpec::toggle(
            "sunlight",
            "gamma_tweaks.setting.sunlight",
            true,
        ));
        let _ = reg.register_setting(SettingSpec::toggle(
            "overlay",
            "gamma_tweaks.setting.overlay",
            false,
        ));
        let _ = reg.register_setting(SettingSpec::toggle(
            "spawnable_only",
            "gamma_tweaks.setting.spawnable_only",
            false,
        ));
        let _ = reg.register_setting(SettingSpec::choice(
            "radius",
            "gamma_tweaks.setting.radius",
            [
                "gamma_tweaks.radius.8",
                "gamma_tweaks.radius.16",
                "gamma_tweaks.radius.24",
                "gamma_tweaks.radius.32",
            ],
            1,
        ));
    }
}
