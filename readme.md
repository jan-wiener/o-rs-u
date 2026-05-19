
## osu! rewritten in Rust, with bevy game engine

## Features:
- Terrible codebase - Bad practices are used often - I wanted to make a compromise on code readablitiy and ease of writing the code. However, success was very limited.
- Actually better performance than osu! (2-4x faster on my hardware)
- Inputs based on rendering frames -> more fps = better input latency (as opposed to osu!, which apparently has a thread for rendering and a separate thread for input - a benefit of building from the ground up without a game engine)
- No GUI for browsing maps, the only way to open custom maps is through the cli

## Additional Information
- For now, UI elements are loaded into the binary at compile time with bevy's embedded assets. I want to include a way to change the skin after compiling, but I will keep the default assets inside the binary.

- There is probably a bug near timing points, where a slider might have different speed than it would have in osu!. This is mostly caused by the lack of documentation for specific things. 
- The scoring system doesn't really work like it should. The score is basically 95% combos.

## Opening a beatmap
```
o-rs-u <path-to-beatmap>
```


## Compiling
Use cargo, build with:
```
cargo build --release
```

Enjoy!
