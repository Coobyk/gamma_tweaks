fn gamma_sky(game: &crate::game::Game<'_>, sky: &mut horizon_client::environment::Sky) {
    crate::game::umbx_gamma_glue::sky(game, sky)
}
