# TWT — Track Window Time

A simple software for X11, written in Rust, to track how much time you spent on each window and save it to a CSV file.


# Summary

After installing the program, running:

```
$ tail ~/.local/share/twt/main.csv
```

will output something like:

|window\_class|window\_name|start|end|
|-------------|------------|-----|---|
|firefox|Russian Dictionary — Mozilla Firefox|1678637136423|1678637137110|
|kitty|bash|1678636643485|1678636667388|
|firefox|chedieck/twt: A software to track the amount of time spend on each window. — Mozilla Firefox|1678636667388|1678636667622|
|kitty|bash|1678636667622|1678636817461|
|Anki|User 1 - Anki|1678636817461|1678636818493|
|kitty|~/codes/twt|1678636818493|1678636820317|

... which is the main table that the software writes. Everything else is meant to be build on top of this simple table.

Installing:
---
```
git clone https://github.com/chedieck/twt.git
cd twt
make install
```

Requires `xdotool` and `playerctl`.


- `make install` sets up the `twt` binary and starts & enables the `systemd` daemon to run it. You can then control it with `systemd --user stop twt`, for example, if you want it to stop recording activity. If you don't use systemd, run `make install-nosystemd` and run the binary as daemon however you like.

Configuring
---
You can set the AFK interval on `~/.config/twt/config.toml`. Default is 60 seconds.
Notice that if you have anything detectable by `playerctl` playing (such as VLC, webbrowser videos, MPV or Spotify), you will not be considered AFK. Check on `playerctl` documentation for the list of software they support


Usage
---
- `twt stat last n 5h` will show the most used windows by name, for the last 5 hours.
- `twt stat span c '2023-03-11 21:50:00'  '2023-03-12 14:30:00'` will show the most used windows by class , from March 11 of 2023 21:50 UTC until March 12 of 2023 14:30 UTC. **Emphasis on UTC**.
- `twt help` for more.



TODO
---
- [x] Basic stats
- [x] Avoid having two running instances
- [x] Better error messages for arg parsing
- [x] AFK detector
- [ ] Allow regex tagging on window name
