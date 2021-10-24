use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::EST;

mod api;
mod config;
mod parser;

use config::Config;


// struct Score {
//     name: String,
//     year: i32,
//     days: [Option<parser::Day>; 25],
// }
// 
// fn calc_score(year: i32, day: u32, completed: &DateTime<Utc>) -> usize {
//     let release_time = EST.ymd(year, 12, day);
//     std::cmp::max(
//         10,
//         50 - 2
//             * (release_time - completed.with_timezone(&EST).date())
//                 .num_days()
//                 .abs(),
//     ) as usize
// }
// 
// impl Score {
//     fn from_leaderboard(leaderboard: parser::Leaderboard) -> Result<Vec<Self>> {
//         let year = leaderboard.event.parse::<i32>()?;
//         let mut scores = Vec::new();
//         for (_, member) in leaderboard.members.into_iter() {
//             let mut days: [Option<parser::Day>; 25] = Default::default();
//             for (day, progress) in member.completion_day_level.into_iter() {
//                 let i: usize = (day.parse::<isize>()? - 1) as usize;
//                 days[i] = Some(progress);
//             }
//             scores.push(Score {
//                 name: member.name.unwrap_or(format!("Anonymous #{}", member.id)),
//                 year,
//                 days,
//             });
//         }
// 
//         // Sort scores in descending order and order by name if tied
//         scores.sort_unstable_by(|a, b| score_sort_key(a).cmp(&score_sort_key(b)));
// 
//         Ok(scores)
//     }
// 
//     fn total(&self) -> usize {
//         let mut score = 0;
//         for (i, day) in self.days.iter().enumerate() {
//             if let Some(d) = day {
//                 score += calc_score(self.year, i as u32 + 1, &d.part1);
//                 if let Some(part2) = d.part2 {
//                     score += calc_score(self.year, i as u32 + 1, &part2);
//                 }
//             }
//         }
//         score
//     }
// }
// 
// fn score_sort_key<'a>(s: &'a Score) -> (std::cmp::Reverse<usize>, &'a str) {
//     (std::cmp::Reverse(s.total()), &s.name)
// }

fn main() -> Result<()> {
    let config = Config::from_file("config.toml")?;
    dbg!(&config);

    let client = api::Client::new(config.session, config.cache_dir);

    for leaderboard_cfg in config.leaderboard.into_iter() {
        dbg!(leaderboard_cfg.name);
        dbg!(client.fetch(leaderboard_cfg.year, leaderboard_cfg.id));
    }
    // let mut config_str = String::new();
    // std::fs::File::open("config.toml")?.read_to_string(&mut config_str)?;
    // let config: Config = toml::from_str(&config_str)?;

    // for leaderboard_id in config.leaderboards {
    //     let leaderboard = api::Leaderboard::fetch(&config.session, 2020, leaderboard_id)?;
    //     let scores = Score::from_leaderboard(leaderboard)?;

    //     // Print dates in header row
    //     for i in 1..=29 {
    //         if i < 14 {
    //             print!(" ");
    //         } else if i < 24 {
    //             print!("1");
    //         } else {
    //             print!("2");
    //         }
    //     }
    //     println!();
    //     for i in 1..=29 {
    //         if i <= 4 {
    //             print!(" ");
    //         } else {
    //             print!("{}", (i - 4) % 10);
    //         }
    //     }
    //     println!();

    //     // Print leaderboard
    //     let mut last_score = 0;
    //     let mut curr_place = 0;
    //     for (i, tally) in scores.into_iter().enumerate() {
    //         let total = tally.total();

    //         // Check if participant is tied with previous participant. If so reuse place
    //         if total != last_score {
    //             curr_place = i + 1;
    //         }
    //         last_score = total;

    //         print!("{:>2}. ", curr_place);
    //         for day in tally.days.iter() {
    //             let star = match day {
    //                 Some(api::Day { part2: Some(_), .. }) => "\x1b[0;93m*\x1b[0m",
    //                 Some(_) => "\x1b[0;96m*\x1b[0m",
    //                 None => "\x1b[0;90m*\x1b[0m",
    //             };
    //             print!("{}", star);
    //         }
    //         println!(" {:>4} {}", total, tally.name);
    //     }
    // }
    Ok(())
}
