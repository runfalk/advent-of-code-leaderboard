use crate::config::LeaderboardConfig;
use crate::model::{Scoreboard, Stars};
use crate::utils::release_time;

pub fn render_template(leaderboard_cfg: &LeaderboardConfig, scoreboard: &Scoreboard) -> String {
    let mut html = String::new();
    html.push_str(HEADER);

    // Leaderboard title and header
    html.push_str(&leaderboard_cfg.header);
    html.push_str("\n\n");
    html.push_str(&format!(
        r#"<h1>{} <span class="star-first-only">({})</span></h1>"#,
        leaderboard_cfg.name, leaderboard_cfg.year,
    ));
    html.push_str("\n");

    // Leaderboard days header
    let now = chrono::Utc::now();
    html.push_str(r#"    <span class="days">"#);
    html.push_str("\n");
    for day in 1..=25 {
        let day_html = if day >= 10 {
            format!("{}<br>{}", day / 10, day % 10)
        } else {
            format!("<br>{}", day)
        };

        if release_time(leaderboard_cfg.year, day).unwrap() < now {
            html.push_str(&format!(
                "        <a href=\"https://adventofcode.com/{}/day/{}\">{}</a>\n",
                leaderboard_cfg.year, day, day_html
            ));
        } else {
            html.push_str(&format!("        {}\n", day_html));
        }
    }
    html.push_str(r#"    </span>"#);
    html.push_str("\n");

    // Leaderboard stars
    let mut last_score = 0;
    let mut curr_place = 0;
    for (i, member) in scoreboard.scores.iter().enumerate() {
        // Check if participant is tied with previous participant. If so reuse place
        if member.score != last_score {
            curr_place = i + 1;
        }
        last_score = member.score;

        html.push_str(&format!("{:>2}) ", curr_place));
        for day in member.stars {
            html.push_str(&format!(
                "{}",
                match day {
                    Stars::Both => r#"<span class="star-both">*</span>"#,
                    Stars::First => r#"<span class="star-first-only">*</span>"#,
                    Stars::None => r#"<span class="star-none">*</span>"#,
                }
            ));
        }
        html.push_str(&format!(" {:>4}", member.score));
        html.push_str(&format!(" {}\n", member.member.name));
    }
    html.push_str("\n");

    html.push_str(COLOR_LEGEND);
    html.push_str("\n\n");

    html.push_str(&format!(
        r#"For those that are interested you can also check the <a href="https://adventofcode.com/{}/leaderboard/private/view/{}">official leaderboard</a>."#,
        leaderboard_cfg.year,
        leaderboard_cfg.id,
    ));
    html.push_str("\n\n");

    html.push_str(HOW_DOES_IT_WORK);
    html.push_str("\n\n");

    html.push_str("<h2>How to join</h2>");
    html.push_str(r#"Go to <a href="https://adventofcode.com/leaderboard">Advent of Code</a>, sign up and join this private leaderboard using the code"#);
    html.push_str("\n");
    html.push_str(&format!("<code>{}</code>.", leaderboard_cfg.code));
    html.push_str("\n\n");

    html.push_str(IM_STUCK);
    html.push_str("\n\n");

    html.push_str(SCORING);
    html.push_str("\n\n");

    html.push_str(WHY_NOT_OFFICIAL_LEADERBOARD);

    html.push_str(FOOTER);
    html
}

const HEADER: &'static str = r#"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
    <title>Advent of Code</title>

    <style type="text/css">
        body {
            padding: 0;
            background: #0f0f23;
            color: #cccccc;
            font-family: monospace;
            font-size: 1.5em;
            text-align: center;
        }

        .content {
            margin-top: 2em;
            display: inline-block;
            text-align: left;
            white-space: pre;
        }

        a {
            color: #009900;
            text-decoration: none;
        }

        a:hover, a:focus {
            color: #99ff99;
        }

        em {
            color: #ffffff;
            font-style: normal;
            text-shadow: 0 0 5px #ffffff;
        }

        h1, h2 {
            margin: 0;
            font-size: inherit;
            color: #ffffff;
            font-weight: normal;
        }

        h1::before, h2::before {
            content: "--- ";
        }

        h1::after, h2::after {
            content: " ---";
        }

        code {
            margin: 0;
            padding: 0;
            position: relative;
            display: inline-block;
        }

        code::before {
            z-index: -1;
            content: "";
            position: absolute;
            display: block;
            left: -2px;
            right: -2px;
            top: 3px;
            bottom: 0px;
            border: 1px solid #333340;
            background: #10101a;
        }

        .days {
            display: inline-flex;
            white-space: nowrap;
        }

        .days > * {
            display: inline-block;
        }

        .star-none {
            color: #555555;
        }

        .star-first-only {
            color: #9999cc;
        }

        .star-both {
            color: #ffff66;
        }
    </style>
</head>
<body>
<div class="content">"#;

const COLOR_LEGEND: &'static str = r#"<span class="star-both">Gold</span> indicates the user got both stars for that day, <span class="star-first-only">silver</span> means just the first
star, and <span class="star-none">gray</span> means none."#;

const HOW_DOES_IT_WORK: &'static str = r#"
<h2>How does it work?</h2>
Each day at <em>00:00 a.m. EST (UTC -5)</em> two puzzles are released. Each player gets
the same problem (but different input values). The answer is usually a number or
a short string that you can paste into the submission field. This means you can
use <em>any language</em> (or pen and paper if you're a lunatic). When you have solved
the first puzzle for a day you unlock the second one. It's possible to skip
puzzles so <a href="https://www.youtube.com/watch?v=tYzMYcUty6s">don't give up</a> if a particular one is giving you trouble.

Remember that the most important thing is to <em>have fun</em>!"#;

const IM_STUCK: &'static str = r#"<h2>I'm stuck on a puzzle</h2>
There is an <a href="https://www.reddit.com/r/adventofcode/">excellent subreddit</a> where you can get hints."#;

const SCORING: &'static str = r#"<h2>How does the scoring work?</h2>
If you solve the puzzle within <em>24 hours</em> from when it's released you get full
points (<em>50</em>). For each day you lag behind the score decreases by <em>5</em>, but you can
never get less than <em>10 points</em> for solving a puzzle. Therefore the maximum score
is <em>2500</em> and the minimum (assuming you solve all puzzles) is <em>500</em>."#;

const WHY_NOT_OFFICIAL_LEADERBOARD: &'static str = r#"<h2>Why not use the official leaderboard?</h2>
By default Advent of Code scores participants by how quickly they solve the
problem compared to everybody else on the same leaderboard. I dislike this since
it encourages participants to use whatever language they are most comfortable
in and rush the solution.

The scoring for this leaderboard values consistency and <em>allows for everybody to
win</em>."#;

const FOOTER: &'static str = r#"
</div>
</body>
</html>
"#;
