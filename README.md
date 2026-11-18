# Gamma Tweaks

A native mod for [Horizon](https://github.com/horizon-rust/horizon) 26.3:
brightness control and a light-level overlay.

## Features

All options live on the game's **Options → Mods → Gamma Tweaks** page:

- **Gamma** — a `0%`–`300%` slider for the lightmap's gamma. With no value
  saved it follows Video Settings' Brightness.
- **Fullbright** — the game's built-in fullbright, from here.
- **Sunlight** — off removes the sky's light contribution (block light and the
  ambient floor remain; the sky itself is untouched).
- **Light-level overlay** — shows the light level (`max` of block and sky
  light) on top of solid blocks around you: red at `0` where mobs can spawn,
  amber `1–7`, green `8+`. Options for **only where mobs spawn** and an
  `8`/`16`/`24`/`32` block radius.

Settings are Horizon's own per-mod settings, under
`horizon-mods/.horizon-mod-data/gamma_tweaks/`.

## Layout

- `horizon-mod.toml` — the new native mod manifest (API `0.2`, pinned to
  Horizon `0b14f8ac…` and horizon-mixins `e59da909…`).
- `src/lib.rs` — the mod crate: settings registration and the world-draw hook.
- `src/runtime.rs`, `src/glue.rs` — support modules injected into the game's
  `game` module; the state, per-tick sync and sky-light hook.
- `src/mixins/` — the two `[[native.mixin]]` helpers:
  `native_client_tick_pre` (sync) and `end_flash_sky` (sunlight).
- `src/overlay.rs` — the light-level overlay's digits and frame state.

## Building

```sh
horizon-launcher native-build . --trust-native-code --version nightly-2026-11-18-0b14f8a
```

The old `native_frame` hook is gone, so the frame sync now runs on
`native_client_tick_pre`. The old custom settings page and the hand-built mod
listing are gone too: Horizon registers the settings rows from `Hooks::register`
and takes the mod list from the manifest. The world-text API was removed, so
the overlay's numbers are now drawn as seven-segment world lines with at most
128 markers per view.
