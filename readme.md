
## osu! rewritten in Rust, with bevy game engine

## Features:
-Terrible codebase - I could optimize/refactor/rewrite, but I don't want to
-Worse performance than original osu! (without capped FPS)
-Inputs based on rendering fps (thanks, bevy) -> more fps = better input latency
-Currently no gui for setting/importing maps, no way to import maps directly
-Currently only very "debug" feeling style


Enjoy!


## Compiling
Use cargo, build with:
```
cargo build --release
```