use std::collections::HashMap;

use crate::config::{LeaderboardConfig, MemberMetadata};
use crate::model::{Scoreboard, Stars};

pub fn render_template(
    cfg: &LeaderboardConfig,
    metadata: &HashMap<usize, MemberMetadata>,
    scoreboard: &Scoreboard,
) {
    println!("{} ({})", cfg.name, cfg.year);
    println!();

    // Print dates in header row
    let padding = 4;
    for i in 1..=25 + padding {
        if i < 14 {
            print!(" ");
        } else if i + padding < 20 {
            print!("1");
        } else {
            print!("2");
        }
    }
    println!();
    for i in 1..=25 + padding {
        if i <= padding {
            print!(" ");
        } else {
            print!("{}", (i - padding) % 10);
        }
    }
    println!();

    // Print leaderboard
    let mut last_score = 0;
    let mut curr_place = 0;
    for (i, member) in scoreboard.scores.iter().enumerate() {
        // Check if participant is tied with previous participant. If so reuse place
        if member.score != last_score {
            curr_place = i + 1;
        }
        last_score = member.score;

        print!("{:>2}. ", curr_place);
        for day in member.stars {
            print!(
                "{}",
                match day {
                    Stars::Both => "\x1b[0;93m*\x1b[0m",
                    Stars::First => "\x1b[0;96m*\x1b[0m",
                    Stars::None => "\x1b[0;90m*\x1b[0m",
                }
            );
        }
        print!(" {:>4}", member.score);
        print!(" {}", member.member.name);

        if let Some(MemberMetadata {
            repository: Some(repo),
        }) = metadata.get(&member.member.id)
        {
            print!(" ({})", repo);
        }

        println!();
    }
}
