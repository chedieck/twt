# TWT — Track Window Time

A simple software written in Rust to track how much time you spent on each window and save it to a CSV file.


# Summary

After installing the program, running:
```
$ cat ~/.local/share/twt/main.csv
```
will output something like:

|window\_class\_name|window\_name|start|end|
|-------------------|------------|-----|---|
|firefox|Russian Dictionary — Mozilla Firefox|1678637136423|1678637137110|
|kitty|bash|1678636643485|1678636667388|
|firefox|chedieck/twt: A software to track the amount of time spend on each window. — Mozilla Firefox|1678636667388|1678636667622|
|kitty|bash|1678636667622|1678636817461|
|Anki|User 1 - Anki|1678636817461|1678636818493|
|kitty|~/codes/twt|1678636818493|1678636820317|

... which is the main table that the software writes. Everything else is meant to be build on top of this simple table.

Installing:
---
1. Install `cargo` and `xdotool` if they are not already installed using your package manager: e.g. `pacman -S cargo xdotool`
2. Clone the repo, go into the directory and run `make install`.
- `make install` sets up the `twt` binary and starts & enables the `systemd` daemon to run it. You can then control it with `systemd --user stop twt`, for example, if you want it to stop recording activity. If you don't use systemd, run `make install-nosystemd` and run the binary as daemon however you like.

Usage
---
- `twt stat '2023-03-11 21:50:00'  '2023-03-12 14:30:00'` will show the most used windows from March 11 of 2023 21:50 UTC until March 12 of 2023 14:30 UTC. **Emphasis on UTC**.


Motivation
---

There are TONS of software for monitoring window activity out there. Most of them you can find on some bloated comercial website, offering you to download & use their propietary software _for free_ for 30 days, and then you have to pay them with something other than the data about all your software usage, accessed websites, etc.

There are also some nice open-source alternatives, but I haven't had luck with any of them: some would require a system tray (which I don't use nor intend to); some would require too much manual intervention like starting and stopping projects; some would rely to much on GUI; some would just not work because they depended on 70 different Haskell packages and one of them didn't work on my computer...

Of course, I did not try all of them. But I was already tired of having so much trouble on using something that I though should be so simple. I just wanted a software that would record the data of window usage, which is pretty straightforward, and then build stuff on top of this data.

TODO
---
- [ ] Tool for exploring the data (draw plots, see tables, cluster by tags, etc.).
- [ ] Avoid having two running instances
- [ ] AFK detector
- [ ] Better error messages for arg parsing
