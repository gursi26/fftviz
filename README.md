# fftviz
A lightweight, customizable FFT visualizer for audio files. Built with Rust + Bevy.

![Screenshot-2024-03-04-at-24620AM](https://github.com/gursi26/fftviz/assets/75204369/9c2d919c-c28c-4021-84dd-64e86d57ae2f)

![demo](https://github.com/gursi26/fftviz/assets/75204369/93b28c8f-d989-49c0-9ec3-dc3fbf7bb2ca)

# Installation

Install fftviz with cargo.
```
cargo install fftviz
```

# Usage
- Run fftviz with a path to an audio file.
```
fftviz "path/to/audio/file.mp3"
```

- Use the `-h` flag for configuration options
```
fftviz -h
A quick FFT visualizer for audio files

Usage: fftviz [OPTIONS] <FILE_PATH>

Arguments:
  <FILE_PATH>  File path to Audio file

Options:
      --border-size <BORDER_SIZE>
          Border size for each bar [default: 1]
      --border-color <BORDER_COLOR>
          Border color for each bar (in hex) [default: 000000]
      --bar-color <BAR_COLOR>
          Color for each bar (in hex) [default: FF0000]
      --disable-title
          Whether to disable printing
      --text-color <TEXT_COLOR>
          Color for currently playing text (in hex) [default: FFFFFF]
      --font-size <FONT_SIZE>
          Font size of currently playing label [default: 25]
      --background-color <BACKGROUND_COLOR>
          [default: 000000]
  -h, --help
          Print help
  -V, --version
          Print version
```
