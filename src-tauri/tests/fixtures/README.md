The embedding fixtures are generated silence (0.05 seconds, 8 kHz mono), with a title tag so the existing writers can update metadata. No user audio is included. Tests copy them into a temporary directory before writing.

Generated with FFmpeg using `-f lavfi -i anullsrc=r=8000:cl=mono -t 0.05 -metadata title=Fixture`, and `-c:a libmp3lame` or `-c:a flac`.
