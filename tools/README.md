# Fake Music Generator

A Python tool to generate fake music files with randomized metadata for testing purposes.

## Features

- Generate MP3 files with fake metadata (artist, album, title, year, genre, track number)
- Organized folder structure: `Artist/Album/Track - Title.mp3`
- Parallel processing for fast generation
- Silent or white noise audio (small file sizes, typically <100KB)
- Supports 100K+ file generation efficiently

## Installation

```bash
cd tools
pip install -r requirements.txt
```

Optional dependencies:

- `faker` - More realistic artist/album names
- `mutagen` - Proper ID3 metadata tags
- `pydub` - Better MP3 generation

## Usage

Basic usage (1000 files):

```bash
python generate_fake_music.py
```

Generate 100K files:

```bash
python generate_fake_music.py -n 100000 -o ./large_library --workers 8
```

Custom options:

```bash
# Shorter songs (5-15 seconds)
python generate_fake_music.py -n 5000 --duration 5 15

# No external dependencies mode
python generate_fake_music.py -n 10000 --no-faker

# Show every file created
python generate_fake_music.py -n 100 -v

# Generate with synced lyrics files
python generate_fake_music.py -n 500 --lrc
```

## Command Line Arguments

| Argument       | Short | Default      | Description                 |
| -------------- | ----- | ------------ | --------------------------- |
| `--num-files`  | `-n`  | 1000         | Number of files to generate |
| `--output`     | `-o`  | ./fake_music | Output directory            |
| `--duration`   |       | 5 30         | Min/max duration in seconds |
| `--workers`    | `-w`  | 4            | Parallel workers            |
| `--no-faker`   |       | False        | Use simple random names     |
| `--audio-type` |       | silent       | silent or white_noise       |
| `--batch-size` |       | 1000         | Progress update interval    |
| `--verbose`    | `-v`  | False        | Show each file created      |
| `--lrc`        |       | False        | Generate .lrc lyrics files  |

## File Size

Files are typically 20-80KB depending on duration (uses low bitrate MP3).

100,000 files ≈ 4-8GB total storage.
