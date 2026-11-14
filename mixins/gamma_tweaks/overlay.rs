// The light-level overlay: the packed light (`&15` block, `>>4` sky) of the
// space above each solid block near the player, drawn as a number on its top
// face.
use super::native_world_draw::NativeWorldDraw;
use super::umbx_gamma as gamma;
use super::Game;
use horizon_client::text::Text;

/// ARGB marks: red where mobs can spawn, amber when dim, green when lit.
const DARK: u32 = 0xFF_FF_55_55;
const DIM: u32 = 0xFF_FF_BB_33;
const LIT: u32 = 0xFF_55_FF_55;
const OUTLINE: u32 = 0xB0_00_00_00;
/// The numbers' world size: the ~9px glyphs at this scale sit on a block top.
const SCALE: f32 = 0.045;
/// Blocks below and above the player's feet to scan for spawnable spaces.
const BELOW: i32 = 5;
const ABOVE: i32 = 9;

/// `native_world_submit`.
pub(super) fn submit(game: &Game<'_>, draw: &mut NativeWorldDraw) {
    if !gamma::overlay() {
        return;
    }
    let Some(player) = game.player.as_ref() else {
        return;
    };
    let facts = game.res.facts();
    let (px, py, pz) = (
        player.position.x.floor() as i32,
        player.position.y.floor() as i32,
        player.position.z.floor() as i32,
    );
    let r = gamma::radius();
    let ymin = (py - BELOW).max(game.world.shape.min_y + 1);
    let ymax = (py + ABOVE).min(game.world.shape.max_y() - 1);
    let spawnable_only = gamma::spawnable_only();
    let mut shown = 0usize;
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
                draw.push();
                if draw.translate_world([
                    f64::from(x) + 0.5,
                    f64::from(y) + 0.01,
                    f64::from(z) + 0.5,
                ]) {
                    // Flat on the block top, glyph top facing north, centred:
                    // the translate lands in font pixels under the scale.
                    draw.rotate_about([1.0, 0.0, 0.0], 90.0, [0.0, 0.0, 0.0]);
                    draw.scale([SCALE, SCALE, SCALE]);
                    draw.translate([if level < 10 { -2.5 } else { -5.0 }, -4.0, 0.0]);
                    game.native_world_text(
                        draw,
                        &Text::literal(level.to_string()),
                        color,
                        Some(OUTLINE),
                    );
                }
                draw.pop();
                shown += 1;
                if shown >= gamma::MAX_MARKERS {
                    break 'done;
                }
            }
        }
    }
}
