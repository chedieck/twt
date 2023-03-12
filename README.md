# TWT — Track Window Time

A simple software written in Rust to track how much time you spent on each window and save it to a CSV file.

Example
---

Running:
```
$ cat ~/.local/share/twt/main.csv
```
should output something like:

|window\_class\_name|window\_name|start|end|
|-------------------|------------|-----|---|
|firefox|Russian Dictionary — Mozilla Firefox|1678637136423|1678637137110|
|kitty|bash|1678636643485|1678636667388|
|firefox|chedieck/twt: A software to track the amount of time spend on each window. — Mozilla Firefox|1678636667388|1678636667622|
|kitty|bash|1678636667622|1678636817461|
|Anki|User 1 - Anki|1678636817461|1678636818493|
|kitty|~/codes/twt|1678636818493|1678636820317|

Installing:
---
1. Install `cargo` and `xdotool` if they are not already installed using your package manager: e.g. `pacman -S cargo xdotool`
2. Clone the repo, go into the directory and run `make install`.
- `make install` sets up the `twt` binary and start & enable the `systemd` daemon to run it. You can then control it with `systemd --user stop twt`, for example, if you want it to stop recording activity. If you don't use systemd, run `make install-nosystemd` and run the daemon as convenient.

Usage
---
- `twt stat '2023-03-11 21:50:00'  '2023-03-12 14:30:00'` will show the most used windows from March 11 of 2023 21:50 UTC until March 12 of 2023 14:30 UTC. **Emphasis on UTC**.

TODO
---
- [ ] Tool for exploring the data (draw plots, see tables, etc.).
- [ ] Avoid having two running instances
- [ ] AFK detector
