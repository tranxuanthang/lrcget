#!/usr/bin/env python3
"""Generate fake music files with random metadata."""

import argparse
import os
import random
import sys
import io
import wave
import subprocess
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path


def create_silent_wav(duration_seconds: int) -> bytes:
    """Create a silent WAV file."""
    sample_rate = 44100
    num_channels = 2
    sample_width = 2
    num_frames = int(sample_rate * duration_seconds)
    
    buffer = io.BytesIO()
    with wave.open(buffer, 'wb') as wav_file:
        wav_file.setnchannels(num_channels)
        wav_file.setsampwidth(sample_width)
        wav_file.setframerate(sample_rate)
        wav_file.setnframes(num_frames)
        silent_data = bytes(num_channels * sample_width * num_frames)
        wav_file.writeframes(silent_data)
    
    return buffer.getvalue()


def convert_wav_to_mp3_ffmpeg(wav_data: bytes, bitrate: str = "64k") -> bytes:
    """Convert WAV data to MP3 using ffmpeg."""
    result = subprocess.run(
        ['ffmpeg', '-y', '-i', 'pipe:0', '-f', 'mp3', '-b:a', bitrate, 'pipe:1'],
        input=wav_data,
        capture_output=True
    )
    
    if result.returncode != 0:
        raise RuntimeError(f"ffmpeg failed: {result.stderr.decode()}")
    
    return result.stdout


def generate_audio_data(duration_seconds: int) -> bytes:
    """Generate valid MP3 audio data using ffmpeg."""
    wav_data = create_silent_wav(duration_seconds)
    return convert_wav_to_mp3_ffmpeg(wav_data, bitrate="64k")


def create_fake_metadata(faker_instance=None):
    """Generate fake metadata for a music file."""
    if faker_instance is None:
        try:
            from faker import Faker
            faker = Faker()
        except ImportError:
            return create_simple_metadata()
    else:
        faker = faker_instance
    
    return {
        'title': faker.catch_phrase() if random.random() > 0.3 else faker.sentence(nb_words=random.randint(2, 5)).rstrip('.'),
        'artist': faker.name(),
        'album': faker.catch_phrase() if random.random() > 0.5 else f"Album {faker.word().title()}",
        'year': str(random.randint(1950, 2024)),
        'genre': random.choice(['Rock', 'Pop', 'Jazz', 'Classical', 'Electronic', 'Hip-Hop', 'R&B', 'Country', 'Metal', 'Indie']),
        'track': str(random.randint(1, 20)),
    }


def create_simple_metadata():
    """Fallback metadata generation without faker."""
    adjectives = ['Blue', 'Red', 'Dark', 'Light', 'Wild', 'Calm', 'Loud', 'Soft', 'Fast', 'Slow']
    nouns = ['Dream', 'Night', 'Day', 'Song', 'Story', 'Life', 'Love', 'Heart', 'Mind', 'Soul']
    verbs = ['Running', 'Flying', 'Falling', 'Rising', 'Dancing', 'Singing', 'Dreaming', 'Living']
    
    first_names = ['John', 'Jane', 'Alex', 'Sam', 'Chris', 'Jordan', 'Taylor', 'Morgan', 'Casey', 'Riley']
    last_names = ['Smith', 'Johnson', 'Williams', 'Brown', 'Jones', 'Garcia', 'Miller', 'Davis', 'Wilson']
    
    return {
        'title': f"{random.choice(adjectives)} {random.choice(nouns)}",
        'artist': f"{random.choice(first_names)} {random.choice(last_names)}",
        'album': f"{random.choice(verbs)} {random.choice(nouns)}",
        'year': str(random.randint(1950, 2024)),
        'genre': random.choice(['Rock', 'Pop', 'Jazz', 'Classical', 'Electronic', 'Hip-Hop', 'R&B']),
        'track': str(random.randint(1, 15)),
    }


def sanitize_filename(name: str) -> str:
    """Sanitize string for use as filename."""
    invalid_chars = '<>:"/\\|?*'
    for char in invalid_chars:
        name = name.replace(char, '_')
    return name[:100]


def generate_lrc_lines(duration_seconds: int, faker_instance=None) -> str:
    """Generate fake LRC lyrics lines."""
    if faker_instance is None:
        lines = [
            "Blue sky calling",
            "Night falls slowly",
            "Dreams of tomorrow",
            "Heart beats faster",
            "Walking alone",
            "Rivers run deep",
            "Shadows dancing",
            "Light in the dark",
            "Whispers in wind",
            "Endless journey",
        ]
    else:
        lines = [faker_instance.sentence(nb_words=random.randint(3, 6)).rstrip('.') for _ in range(10)]

    num_lines = random.randint(5, min(15, max(5, duration_seconds // 3)))
    selected = [random.choice(lines) for _ in range(num_lines)]

    # Spread lines evenly across the duration
    lrc_lines = []
    for i, line in enumerate(selected):
        seconds = int((i + 1) * duration_seconds / (num_lines + 1))
        minutes, secs = divmod(seconds, 60)
        lrc_lines.append(f"[{minutes:02d}:{secs:02d}.00]{line}")

    return "\n".join(lrc_lines)


def generate_single_file(args) -> tuple:
    """Generate a single fake music file."""
    file_index, output_dir, duration_range, use_faker, generate_lrc = args
    
    try:
        # Generate metadata
        if use_faker:
            from faker import Faker
            faker = Faker()
            metadata = create_fake_metadata(faker)
        else:
            metadata = create_simple_metadata()
        
        # Determine folder structure
        artist_folder = sanitize_filename(metadata['artist'])
        album_folder = sanitize_filename(metadata['album'])
        
        # Create folder path
        file_dir = Path(output_dir) / artist_folder / album_folder
        file_dir.mkdir(parents=True, exist_ok=True)
        
        # Generate filename
        track_num = metadata['track'].zfill(2)
        safe_title = sanitize_filename(metadata['title'])
        filename = f"{track_num} - {safe_title}.mp3"
        filepath = file_dir / filename
        
        # Handle duplicates
        counter = 1
        while filepath.exists():
            filename = f"{track_num} - {safe_title}_{counter}.mp3"
            filepath = file_dir / filename
            counter += 1
        
        # Generate audio
        duration = random.randint(duration_range[0], duration_range[1])
        audio_data = generate_audio_data(duration)
        
        # Write file
        with open(filepath, 'wb') as f:
            f.write(audio_data)
        
        # Generate LRC file if requested
        if generate_lrc:
            lrc_content = generate_lrc_lines(duration, faker if use_faker else None)
            lrc_path = filepath.with_suffix('.lrc')
            with open(lrc_path, 'w', encoding='utf-8') as f:
                f.write(lrc_content)
        
        # Add metadata using mutagen
        try:
            from mutagen.mp3 import MP3
            from mutagen.id3 import ID3, TIT2, TPE1, TALB, TDRC, TCON, TRCK
            
            audio = MP3(filepath)
            if audio.tags is None:
                audio.add_tags()
            
            audio.tags['TIT2'] = TIT2(encoding=3, text=metadata['title'])
            audio.tags['TPE1'] = TPE1(encoding=3, text=metadata['artist'])
            audio.tags['TALB'] = TALB(encoding=3, text=metadata['album'])
            audio.tags['TDRC'] = TDRC(encoding=3, text=metadata['year'])
            audio.tags['TCON'] = TCON(encoding=3, text=metadata['genre'])
            audio.tags['TRCK'] = TRCK(encoding=3, text=metadata['track'])
            
            audio.save()
        except ImportError:
            pass
        
        return (True, str(filepath), None)
        
    except Exception as e:
        return (False, None, str(e))


def main():
    parser = argparse.ArgumentParser(
        description='Generate fake music files with random metadata',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  python generate_fake_music.py -n 1000 -o ./fake_music
  python generate_fake_music.py -n 100000 --duration 5 15 --workers 8
  python generate_fake_music.py -n 5000 --no-faker
        '''
    )
    
    parser.add_argument(
        '-n', '--num-files',
        type=int,
        default=1000,
        help='Number of files to generate (default: 1000)'
    )
    
    parser.add_argument(
        '-o', '--output',
        type=str,
        default='./fake_music',
        help='Output directory for generated files (default: ./fake_music)'
    )
    
    parser.add_argument(
        '--duration',
        type=int,
        nargs=2,
        metavar=('MIN', 'MAX'),
        default=[5, 30],
        help='Duration range in seconds (default: 5 30)'
    )
    
    parser.add_argument(
        '-w', '--workers',
        type=int,
        default=4,
        help='Number of parallel workers (default: 4)'
    )
    
    parser.add_argument(
        '--no-faker',
        action='store_true',
        help='Use simple random generation instead of Faker library'
    )
    
    parser.add_argument(
        '--batch-size',
        type=int,
        default=1000,
        help='Number of files to process before showing progress (default: 1000)'
    )
    
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Show detailed progress'
    )
    
    parser.add_argument(
        '--lrc',
        action='store_true',
        help='Also generate a corresponding .lrc file next to each music file'
    )
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.num_files <= 0:
        print("Error: Number of files must be positive", file=sys.stderr)
        sys.exit(1)
    
    if args.duration[0] > args.duration[1] or args.duration[0] < 1:
        print("Error: Invalid duration range", file=sys.stderr)
        sys.exit(1)
    
    # Check ffmpeg is available
    result = subprocess.run(['which', 'ffmpeg'], capture_output=True)
    if result.returncode != 0:
        print("Error: ffmpeg is required but not found in PATH", file=sys.stderr)
        sys.exit(1)
    
    # Create output directory
    output_path = Path(args.output)
    output_path.mkdir(parents=True, exist_ok=True)
    
    # Check for optional dependencies
    use_faker = not args.no_faker
    if use_faker:
        try:
            from faker import Faker
            print("✓ Faker library available - using realistic metadata")
        except ImportError:
            print("⚠ Faker not installed - using simple random metadata")
            print("  Install with: pip install faker")
            use_faker = False
    
    try:
        from mutagen.mp3 import MP3
        print("✓ Mutagen library available - ID3 tags will be written")
    except ImportError:
        print("⚠ Mutagen not installed - files will have no metadata tags")
        print("  Install with: pip install mutagen")
    
    print(f"\nGenerating {args.num_files:,} fake music files...")
    print(f"Output directory: {output_path.absolute()}")
    print(f"Duration range: {args.duration[0]}-{args.duration[1]} seconds")
    print(f"Workers: {args.workers}")
    print()
    
    # Prepare work items
    work_items = [
        (i, str(output_path), args.duration, use_faker, args.lrc)
        for i in range(args.num_files)
    ]
    
    # Generate files with progress tracking
    success_count = 0
    error_count = 0
    
    with ProcessPoolExecutor(max_workers=args.workers) as executor:
        futures = {executor.submit(generate_single_file, item): item for item in work_items}
        
        for i, future in enumerate(as_completed(futures)):
            success, filename, error = future.result()
            
            if success:
                success_count += 1
                if args.verbose:
                    print(f"Created: {filename}")
            else:
                error_count += 1
                if args.verbose:
                    print(f"Error: {error}")
            
            # Progress update
            if (i + 1) % args.batch_size == 0:
                print(f"Progress: {i + 1:,}/{args.num_files:,} files ({success_count} OK, {error_count} errors)")
    
    # Final summary
    print(f"\n{'='*50}")
    print(f"Generation complete!")
    print(f"Total files: {args.num_files:,}")
    print(f"Successful:  {success_count:,}")
    print(f"Errors:      {error_count:,}")
    print(f"Output:      {output_path.absolute()}")
    print(f"{'='*50}")
    
    if error_count > 0:
        sys.exit(1)


if __name__ == '__main__':
    main()
