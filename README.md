Advent of Code Leaderboard
==========================
This repository provides an alternative leaderboard scoring for
[Advent of Code](https://adventofcode.com/).

By default Advent of Code scores participants by how quickly they solve the
problem compared to everybody else on the same leaderboard. I dislike this since
it encourages participants to use whatever language they are most comfortable
in and rush the solution.

The scoring for this leaderboard values consistency and allows for everybody to
win.


Usage
-----
```
# Print all leaderboard in the terminal with colors (great for testing)
advent-of-code-leaderboard console config.toml

# Host an HTML version of the leaderboard on http://localhost:3000/your-leaderboard-slug
advent-of-code-leaderboard server config.toml
```

The configuration file has the following structure:

```toml
# This needs to be obtained from your session cookie (96 character hex string)
session = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"

# Directory where cached JSON API responses are saved (we're only allowed to
# refresh once every 15 minutes). If this isn't set it'll try to find a
# suitable cache directory on its own
cache_dir = "./"

# You can define an arbitrary number of leaderboards
[[leaderboard]]
id = 0  # Unique ID of your leaderboard
year = 2021
name = "Name of your leaderboard"
slug = "leaderboard-slug"  # This one determines the access URL
code = "000000-00000000"  # Leaderboard join code

# Optional header to display on top of the leaderboard. Put some pretty ASCII
# art here :)
header = """
  .-""-.
 /,..___\
() {_____}
  (/-@-@-\)
  {`-=^=-'}
  {  `-'  }
   {     }
    `---
"""

# Add additional metadata to the leaderboard. The number is member ID
[[metadata]]
year = 2021
273465 = { repository = "https://github.com/runfalk/advent-of-code-2021/" }
```
