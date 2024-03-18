# fftviz
A lightweight, customizable FFT visualizer for audio files. Built with Rust + Bevy.

![Presentation1](https://github.com/gursi26/fftviz/assets/75204369/590f0021-1aa3-44a1-9396-c0daf4af4080)

https://github.com/gursi26/fftviz/assets/75204369/b53589f3-9871-416e-96df-c9de2591f835


# Installation

## Cargo
```
cargo install fftviz
```

## Homebrew
```
brew tap gursi26/fftviz
brew install fftviz
```

# Keybinds
- `q` to close window.
- `e` to open config gui in player window.
- `Space` to pause/play.
- `↑` to increase volume.
- `↓` to decrease volume.

# Usage
- Run fftviz with a path to an audio file.
```
fftviz "path/to/audio/file.mp3"
```

- Run with `-h` flag for configuration options
```
fftviz -h
A lightweight, customizable FFT visualizer for audio files

Usage: fftviz [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>  File path to Audio file

Options:
      --smoothness <SMOOTHNESS>
          Smoothing factor for spatial interpolation between bars
      --freq-resolution <FREQ_RESOLUTION>
          Number of individual frequencies detected by the FFT
      --min-freq <MIN_FREQ>
          Maximum frequency detected by FFT
      --max-freq <MAX_FREQ>
          Minimum frequency detected by FFT
      --volume <VOLUME>
          Volume
      --width <WINDOW_WIDTH>
          Window width
      --height <WINDOW_HEIGHT>
          Window height
      --border-size <BORDER_SIZE>
          Border size for each bar
      --border-color <BORDER_COLOR>
          Border color for each bar (in hex)
      --bar-color <BAR_COLOR>
          Color for each bar (in hex)
      --track-name
          Use if you want track name to be printed
      --display-gui
          Use if you want the gui to be open when launched
      --debug
          Use if you want to display debug information when launching
      --no-title-bar
          Use to disable the title bar
      --text-color <TEXT_COLOR>
          Color for currently playing text (in hex)
      --font-size <FONT_SIZE>
          Font size of currently playing label
      --background-color <BACKGROUND_COLOR>

  -h, --help
          Print help
  -V, --version
          Print version
```
