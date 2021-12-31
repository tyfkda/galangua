Galangua
========

2D shoot 'em up game, written in Rust.

[![screenshot](ss.png)](https://tyfkda.github.io/galangua/)

[Play in browser](https://tyfkda.github.io/galangua/)

### How to play (Control)

  * Arrow key : Move left or right
  * Space key : Shoot a bullet


### Requirement

  * Rust, Cargo
  * SDL2

#### MacOS

  * `brew install sdl2 sdl2_image sdl2_ttf sdl2_mixer`
  * Set SDL2, SDL2_image, SDL2_mixer lib paths to LIBRARY_PATH environment variable.

```bash
SDL2=/opt/homebrew/Cellar/sdl2/2.0.X
SDL2_IMAGE=/opt/homebrew/Cellar/sdl2_image/2.0.X
SDL2_MIXER=/opt/homebrew/Cellar/sdl2_mixer/2.0.X
export LIBRARY_PATH="$LIBRARY_PATH:$SDL2/lib:$SDL2_IMAGE/LIB:$SDL2_MIXER/lib"
```

#### Windows : Install SDL2 libraries

  * Download `SDL2-devel-2.0.x-VC.zip` from [SDL2](https://www.libsdl.org/),
    `SDL2_image-devel-2.0.x-VC.zip` from [SDL2_image](https://www.libsdl.org/projects/SDL_image/),
    and `SDL2_mixer-devel-2.0.x-VC.zip` from [SDL2_mixer](https://www.libsdl.org/projects/SDL_mixer/) libraries
  * Unpack zip files and copy libraries into `C:\Users\{Your Username}\.rustup\toolchains\{current toolchain}\lib\rustlib\{current toolchain}\lib`
    * See [README](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc)

### Build

    $ cargo build --release

### Run

    $ cargo run --release

#### Command-line options

  * -s <scale> : Specify window scale (default: 3)
  * -f         : Use fullscreen
  * --oo       : Run object-oriented version


### Browser version

#### Requirement

  * [wasm-pack](https://rustwasm.github.io/wasm-pack/)

#### Build

    $ cd galangua-wasm
    $ make  # wasm-pack build
    $ make start-server  # Start local server on port 8080

#### Release build

    $ cd www
    $ npm install
    $ npm run build

Files are generated in `galangua-wasm/www/dist`


### Assets

  * SE
    * 効果音は[スキップモア](https://www.skipmore.com/)の物を使用しています
