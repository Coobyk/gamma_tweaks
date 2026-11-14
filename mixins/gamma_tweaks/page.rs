// The mod's Horizon Settings page: the slider a flat native-mod page cannot
// draw. Registered on the first frame — the page registry seals when the
// options screens first read it, which a frame always beats. Rows come from
// `runtime`'s statics, which `sync` keeps loaded, applies and persists.
use super::umbx_gamma as gamma;
use horizon_hooks::pages::{Act, Label, Page, Place, Row};
use std::sync::Once;

struct GammaPage;

static PAGE: GammaPage = GammaPage;

/// Add the page to Horizon Settings, once.
pub(super) fn register() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = horizon_hooks::pages::register(&PAGE);
    });
}

fn pct(v: f32) -> String {
    format!("{}%", (v * 100.0).round() as i32)
}

impl Page for GammaPage {
    fn id(&self) -> &'static str {
        "gamma_tweaks.page"
    }
    fn title(&self) -> &'static str {
        "Gamma Tweaks"
    }
    fn place(&self) -> Place {
        Place::HorizonSettings
    }
    fn rows(&self, rows: &mut Vec<Row>) {
        let (override_gamma, effective, fullbright, sunlight, overlay, spawnable, radius) =
            gamma::page_state();
        rows.push(Row::Heading("Gamma"));
        rows.push(Row::Slider {
            label: "Gamma",
            value: (effective / gamma::MAX_GAMMA).clamp(0.0, 1.0),
            text: Label::Value(if override_gamma.is_some() {
                pct(effective)
            } else {
                format!("{} (video)", pct(effective))
            }),
        });
        rows.push(Row::Toggle {
            label: "Follow video Brightness",
            on: override_gamma.is_none(),
        });
        rows.push(Row::Heading("Light"));
        rows.push(Row::Toggle {
            label: "Fullbright",
            on: fullbright,
        });
        rows.push(Row::Toggle {
            label: "Sunlight",
            on: sunlight,
        });
        rows.push(Row::Heading("Light-level overlay"));
        rows.push(Row::Toggle {
            label: "Show light levels",
            on: overlay,
        });
        rows.push(Row::Toggle {
            label: "Only where mobs spawn",
            on: spawnable,
        });
        rows.push(Row::Cycle {
            label: "Radius",
            value: Label::Value(format!("{radius} blocks")),
        });
    }
    fn act(&self, row: usize, act: Act) {
        match (row, act) {
            (1, Act::Slide(v)) => {
                gamma::set_gamma(Some((v * gamma::MAX_GAMMA).clamp(0.0, gamma::MAX_GAMMA)));
            }
            (2, Act::Press) => gamma::set_gamma(match gamma::gamma() {
                Some(_) => None,
                None => Some(gamma::effective_gamma()),
            }),
            (4, Act::Press) => gamma::set_fullbright(!gamma::fullbright()),
            (5, Act::Press) => gamma::set_sunlight(!gamma::sunlight()),
            (7, Act::Press) => gamma::set_overlay(!gamma::overlay()),
            (8, Act::Press) => gamma::set_spawnable_only(!gamma::spawnable_only()),
            (9, Act::Press) => gamma::cycle_radius(false),
            (9, Act::PressBack) => gamma::cycle_radius(true),
            _ => {}
        }
    }
    fn tooltip(&self, row: usize) -> Option<Label> {
        Some(Label::Value(
            match row {
                1 => "0% is fully dark; 300% is three times Bright.",
                2 => "Use Video Settings' Brightness again.",
                4 => "Everywhere as bright as possible.",
                5 => "Off keeps block light but removes the sky's.",
                7 => "The light level above each solid block around you.",
                8 => "Only mark light 0, where mobs can spawn.",
                _ => return None,
            }
            .to_owned(),
        ))
    }
}
