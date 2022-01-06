use minijinja::{context, Environment, State};
use serde::Serialize;
use std::collections::HashMap;

use crate::config::{LeaderboardConfig, MemberMetadata};
use crate::model::{Scoreboard, Stars};
use crate::utils::release_time;

const TEMPLATE: &str = include_str!("leaderboard.html");

#[derive(Debug, Serialize)]
struct LeaderboardLine<'a> {
    place: usize,
    score: usize,
    star_classes: [&'a str; 25],
    name: &'a str,
    repository: &'a str,
}

fn chars(_state: &State, value: String) -> Result<Vec<char>, minijinja::Error> {
    Ok(value.chars().collect())
}

fn is_unlocked(_state: &State, year: i32, day: u32) -> Result<bool, minijinja::Error> {
    let rt = release_time(year, day).map_err(|_| {
        minijinja::Error::new(minijinja::ErrorKind::InvalidArguments, "Invalid date")
    })?;
    Ok(rt < chrono::Utc::now())
}

fn left_pad(_state: &State, value: String, width: usize) -> Result<String, minijinja::Error> {
    Ok(std::iter::repeat(' ')
        .take(width.saturating_sub(value.len()))
        .chain(value.chars())
        .collect())
}

pub fn render_template(
    cfg: &LeaderboardConfig,
    metadata: &HashMap<usize, MemberMetadata>,
    scoreboard: &Scoreboard,
) -> String {
    let mut env = Environment::new();
    env.add_filter("chars", chars);
    env.add_filter("left_pad", left_pad);
    env.add_function("is_unlocked", is_unlocked);
    env.add_template("template", TEMPLATE).unwrap();
    let tmpl = env.get_template("template").unwrap();

    let mut last_score = 0;
    let mut curr_place = 0;
    let mut leaderboard = Vec::new();
    for (i, member) in scoreboard.scores.iter().enumerate() {
        // Check if participant is tied with previous participant. If so reuse place
        if member.score != last_score {
            curr_place = i + 1;
        }
        last_score = member.score;

        let repository = metadata
            .get(&member.member.id)
            .and_then(|m| m.repository.as_deref())
            .unwrap_or("");
        leaderboard.push(LeaderboardLine {
            place: curr_place,
            star_classes: member
                .stars
                .iter()
                .map(|s| match s {
                    Stars::None => "star-none",
                    Stars::First => "star-first-only",
                    Stars::Both => "star-both",
                })
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            score: member.score,
            name: &member.member.name,
            repository,
        });
    }

    tmpl.render(context!(
        name => cfg.name,
        year => cfg.year,
        header => cfg.header,
        code => cfg.code,
        leaderboard => leaderboard,
    ))
    .unwrap()
}
