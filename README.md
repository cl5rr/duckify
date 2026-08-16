<img src="assets/logo.png" alt="Duckify" width="220">

# Duckify

Your music gets quiet on its own when something else needs to be heard, then
comes back when it is done.

Start a game and Spotify pauses. Someone talks in Discord and the music dips for
a moment. Open a YouTube video and it steps aside. Nothing to press.

**Windows only.**

## Install

### Step 1: get Spicetify

Duckify puts a button inside Spotify, and that needs Spicetify first.

**[Install Spicetify here](https://spicetify.app/)** and follow their
instructions. Run it once, then come back.

Already have Spicetify? Skip to step 2.

### Step 2: run the Duckify installer

1. Download **Duckify-Setup.exe** from the
   [latest release](https://github.com/heybu/duckify/releases/latest)
2. Double click it
3. Press **Install**

That is the whole thing. The installer sets everything up, starts Duckify, and
makes it come back on its own whenever you turn your computer on.

> **Windows may show a blue "Windows protected your PC" box.** That appears for
> any program without a paid certificate. Click **More info**, then
> **Run anyway**.

### Step 3: look for the duck

Open Spotify. There is a small duck icon in the top bar, next to the shopping
bag. Click it to change how Duckify behaves.

## Removing it

Run the same installer again. It notices Duckify is already there and offers
**Remove** instead.

## What it does, exactly

| What is happening | What your music does |
|---|---|
| A game is making sound | Pauses |
| A game is open but quiet | Keeps playing softly |
| Someone is talking in a call | Dips for a moment, then comes back |
| A video is playing in your browser | Pauses |
| Nothing else making noise | Normal |

Every one of these is adjustable, and each can be switched off.

Quieting is relative to whatever volume you picked. If you listen at 60%, a dip
to 10% means a tenth of *your* 60%, not a tenth of maximum. Change the volume
yourself at any time and Duckify treats that as your new normal.

## Common questions

**Do I have to keep something open?**
No. A small background program does the listening. It starts with Windows and
uses almost nothing while idle.

**Will it pause for the wrong things?**
It only reacts to programs it recognises, plus anything you approve. When
something unfamiliar makes noise, it asks you once and remembers your answer.
Until you answer, it does nothing.

**Something is pausing my music and I want it to stop.**
Open the duck menu in Spotify. Unrecognised programs are listed there with Yes
and No buttons, and there is a **Reset decisions** link at the bottom if you
want to start over.

**Can I turn it off for a while?**
Yes. The duck menu has an **Enabled** switch.

**It says "Helper not running".**
The background program is not started. Turn on **Start with Windows** in the
menu, or run the installer again.

## For developers

Duckify is two pieces, because Spotify's window cannot see other programs:

- `helper/` is a small Rust program that watches per-application audio through
  WASAPI and decides what should happen
- `extension/` is the Spicetify extension that applies those decisions and draws
  the settings panel

They talk over a WebSocket on `127.0.0.1:8787`, which never leaves the machine.

```
cd helper
cargo test --bin duckify-helper   # rule engine and game detection
cargo build --release
cargo run --example scan          # what game detection finds here
cargo run --example peaks         # live per-application audio levels

cd ../installer
cargo build --release             # embeds the helper and the extension
```

`examples/peaks.rs` is the one to reach for when something ducks that should
not: it prints exactly which programs Windows reports as making sound.

### How games are detected

There is no "is this a game" API on Windows, so several signals are combined:

1. **Steam library manifests**, including libraries on other drives
2. **A short built-in list** for games no launcher knows about, such as Roblox,
   which runs every experience through one executable
3. **Anything unrecognised is never acted on.** It is logged, you are asked once,
   and the answer is permanent
4. **A denylist** for browsers, recording software, and media players, which an
   explicit approval can override

Some names are deliberately never guessed. `javaw.exe` is Minecraft, but it is
also every other Java program, so it always asks.

### Why the timings matter

A game that is mostly quiet with occasional sounds would otherwise flip between
paused and playing constantly. Sound must persist briefly before Duckify reacts,
and must stay gone before it comes back. Voice uses the same idea with much
shorter timings, because speech has natural gaps between words.

## Licence

MIT.
