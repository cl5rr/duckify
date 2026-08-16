#[path = "../src/games.rs"]
mod games;

fn main() {
    let index = games::GameIndex::build(&[], &[]);
    println!("indexed executables: {}", index.known_count());

    for probe in [
        "robloxplayerbeta.exe",
        "javaw.exe",
        "chrome.exe",
        "spotify.exe",
        "discord.exe",
        "mystery-indie-game.exe",
    ] {
        println!("  {probe:<28} -> {:?}", index.classify(probe));
    }
}
