# fftviz
A lightweight, customizable FFT visualizer for audio files. Built with Rust + Bevy.

![Screenshot-2024-03-04-at-24620AM](https://github.com/gursi26/fftviz/assets/75204369/9c2d919c-c28c-4021-84dd-64e86d57ae2f)

![demo](https://github.com/gursi26/fftviz/assets/75204369/93b28c8f-d989-49c0-9ec3-dc3fbf7bb2ca)

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

# Keybinds:
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
      --fft-fps <FFT_FPS>
          Temporal resolution for FFT calculation (rendering always occurs at 60 fps with interpolation) [default: 12]
      --smoothness <SMOOTHNESS>
          Smoothing factor for spatial interpolation between bars [default: 1]
      --freq-resolution <FREQ_RESOLUTION>
          Number of individual frequencies detected by the FFT [default: 90]
      --min-freq <MIN_FREQ>
          Maximum frequency detected by FFT [default: 0]
      --max-freq <MAX_FREQ>
          Minimum frequency detected by FFT [default: 5000]
      --volume <VOLUME>
          Volume [default: 0.7]
      --width <WINDOW_WIDTH>
          Window width [default: 1000]
      --height <WINDOW_HEIGHT>
          Window height [default: 700]
      --border-size <BORDER_SIZE>
          Border size for each bar [default: 1]
      --border-color <BORDER_COLOR>
          Border color for each bar (in hex) [default: 000000]
      --bar-color <BAR_COLOR>
          Color for each bar (in hex) [default: FF0000]
      --track-name
          Use if you want track name to be printed
      --display-gui
          Use if you want the gui to be open when launched
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
