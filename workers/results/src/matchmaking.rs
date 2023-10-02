use std::error::Error;

use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl};
use rand::Rng;
use shared::db::{
    conn::DB_CONNECTION,
    dao::games::GamesDao,
    models::{Bot, Team},
    schema,
};

pub async fn matchmake(s3_client: &aws_sdk_s3::Client, sqs_client: &aws_sdk_sqs::Client) {
    log::info!("starting matchmaking");
    let mut player_count = None;
    loop {
        log::info!("matchmaking");
        // load all teams
        match matchmake_round(&s3_client, &sqs_client).await {
            Err(e) => {
                log::error!("Error in matchmake_round: {:?}", e);
            }
            Ok(updated_player_count) => {
                player_count = Some(updated_player_count as u64);
            }
        }
        // give an average of 10 seconds per game, plus 10
        tokio::time::sleep(std::time::Duration::from_secs(
            (10 + player_count.unwrap_or(10)) * 10,
        ))
        .await;
    }
}

pub async fn matchmake_round(
    s3_client: &aws_sdk_s3::Client,
    sqs_client: &aws_sdk_sqs::Client,
) -> Result<usize, Box<dyn Error>> {
    let db_conn = &mut (*DB_CONNECTION).get().unwrap();
    let teams: Vec<Team> = schema::teams::table
        .order_by(schema::teams::dsl::rating)
        .load::<Team>(db_conn)?;

    let mut rng = rand::thread_rng();
    if teams.len() == 1 {
        return Ok(1);
    }
    for i in 0..teams.len() {
        log::debug!("Matchmaking for team {}", teams[i].name);
        // pick another team to play against, randomly within 5 indices
        let mut other = rng.gen_range(i.saturating_sub(5)..(i + 6).min(teams.len()));
        while other == i {
            // don't play against yourself
            other = rng.gen_range(i.saturating_sub(5).max(0)..(i + 6).min(teams.len()));
        }
        let other = &teams[other];
        let this = &teams[i];
        match db_conn
            .create_game(
                other,
                this,
                true,
                &*std::env::var("GAME_LOGS_S3_BUCKET").unwrap(),
                &*std::env::var("NEW_GAMES_QUEUE_URL").unwrap(),
                &sqs_client,
                &s3_client,
            )
            .await
        {
            Ok(_) => {
                log::info!("Created game between {} and {}", this.name, other.name);
            }
            Err(_) => {
                log::info!(
                    "Failed to create game between {} and {}",
                    this.name,
                    other.name
                );
            }
        }
    }

    log::info!("teams: {:?}", teams);
    Ok(teams.len())
}
