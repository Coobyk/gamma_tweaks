# Gamma Tweaks

A [Horizon](https://github.com/horizon-rust/horizon) native mod for brightness
control and a light-level overlay.

## Features

A settings page on **Horizon Settings → Gamma Tweaks** (also reachable as
buttons under **Options → Mods → Gamma Tweaks**):

- **Gamma** — a slider from `0%` (fully dark) to `300%` (three times vanilla
  Bright), synced with the real Brightness option. **Follow video Brightness**
  hands control back to the vanilla slider.
- **Fullbright** — the game's built-in fullbright, from here.
- **Sunlight** — off removes the sky's light contribution (block light and the
  ambient floor remain; the sky itself is untouched).
- **Light-level overlay** — draws the light level (`max` of block and sky
  light) on the top face of solid blocks around you: red at `0` where mobs
  can spawn, amber `1–7`, green `8+`. Options for **only where mobs spawn**
  and a radius of `8`/`16`/`24`/`32` blocks.

Settings persist per instance under `.horizon-mod-data/gamma_tweaks/`.

## Layout

A mixin bundle (`mixins/gamma_tweaks/`, `horizon-mixin.toml`):

- `runtime.rs` — the mod's state: a per-frame `sync` loads it, applies the
  gamma override to `settings.gamma`/`renderer.gamma` and the fullbright flag
  to `settings.extras.fullbright`, and persists changes.
- `page.rs` — a registered `horizon_hooks::pages::Page` (the slider rows);
  the engine rebuilds them from the runtime's statics.
- `overlay.rs` — `native_world_submit` draws the numbers as world text.
- `glue.rs` — hook glue: `native_frame` (register + sync), `end_flash_sky`
  (sunlight), `native_world_submit`, the mod listing and the buttons page.
- `hooks/` — one thin wrapper per injection.

## Building

```sh
horizon-mixins plan --workspace <horizon source> --mods mixins/gamma_tweaks --out plan.json
horizon-mixins apply --workspace <horizon source> --mods mixins/gamma_tweaks --out <staged>
```

or drop the bundle folder on `horizon-launcher native-build <folder>
--trust-native-code`. Target revision in `compatibility.json`.
