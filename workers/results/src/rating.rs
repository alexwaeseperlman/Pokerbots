const K_FACTOR: f32 = 32.0;
/// elo rating system
pub fn get_rating_change(
    defender_rating: f32,
    defender_score: f32,
    challenger_rating: f32,
    challenger_score: f32,
) -> (f32, f32) {
    let defender_expected =
        1.0 / (1.0 + 10.0f32.powf((challenger_rating - defender_rating) / 400.0));
    let challenger_expected =
        1.0 / (1.0 + 10.0f32.powf((defender_rating - challenger_rating) / 400.0));
    let defender_change = K_FACTOR * (defender_score - defender_expected);
    let challenger_change = K_FACTOR * (challenger_score - challenger_expected);

    log::info!(
        "defender_rating: {}, defender_score: {}, challenger_rating: {}, challenger_score: {}, defender_change: {}, challenger_change: {}",
        defender_rating,
        defender_score,
        challenger_rating,
        challenger_score,
        defender_change,
        challenger_change
    );
    (defender_change, challenger_change)
}
