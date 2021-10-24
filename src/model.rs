use std::cmp;

use crate::parser::{Day, Leaderboard};
use crate::utils::{release_time, score_puzzle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stars {
    None,
    First,
    Both,
}

#[derive(Debug)]
pub struct Member {
    pub id: usize,
    pub name: String,
    pub github: Option<String>,
}

#[derive(Debug)]
pub struct MemberScore {
    pub member: Member,
    pub stars: [Stars; 25],
    pub score: usize,
}

#[derive(Debug)]
pub struct Scoreboard {
    pub year: i32,
    pub scores: Vec<MemberScore>,
}

impl Scoreboard {
    pub fn from_leaderboard(leaderboard: &Leaderboard) -> Self {
        let year = leaderboard.event;
        let mut scores: Vec<_> = leaderboard
            .members
            .values()
            .map(|leaderboard_member| {
                let member = Member {
                    id: leaderboard_member.id,
                    name: leaderboard_member
                        .name
                        .as_ref()
                        .map(ToOwned::to_owned)
                        .unwrap_or_else(|| format!("(anonymous user #{})", leaderboard_member.id)),
                    github: None,
                };

                let (stars, score) = (1..=25usize).into_iter().fold(
                    ([Stars::None; 25], 0),
                    |(mut stars, mut score), day| {
                        let released = release_time(year, day as u32).unwrap();
                        let parts = leaderboard_member.completion_day_level.get(&day);

                        score += match parts {
                            Some(Day {
                                part1,
                                part2: Some(part2),
                            }) => {
                                stars[day - 1] = Stars::Both;
                                score_puzzle(*part1 - released) + score_puzzle(*part2 - released)
                            }
                            Some(Day { part1, part2: None }) => {
                                stars[day - 1] = Stars::First;
                                score_puzzle(*part1 - released)
                            }
                            None => 0,
                        };

                        (stars, score)
                    },
                );

                MemberScore {
                    member,
                    stars,
                    score,
                }
            })
            .collect();

        // Use ID as a discirminator to ensure deterministic result
        scores.sort_by_key(|member| (cmp::Reverse(member.score), member.member.id));

        // Sort
        Self { year, scores }
    }
}
