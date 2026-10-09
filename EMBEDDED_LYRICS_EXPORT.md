# Embedded Lyrics Export Improvements

Contribution by **Rick Lidgett** (GitHub: `scriptusscript777`).

## Changes

- The lyrics editor initially selects **Synced lyrics (.lrc)**, making sidecar
  export available without reselecting the checkbox every time the editor opens.
- **Embed into track** initially follows the saved `export_embedded` preference
  only when the experimental `try_embed_lyrics` setting is enabled.
- Reopening the export menu refreshes that experimental setting without resetting
  the user's current checkbox choices.
- A disabled embedding option cannot enable the export button by itself and is
  excluded from the submitted export request.
- MP3 export removes older USLT and SYLT language/description variants before
  inserting the selected lyrics. Previously, an old `eng` frame could coexist
  with the newly exported `XXX` frame, allowing a player to display stale lyrics.
- Repeated embedded exports leave one plain and one synchronized lyric frame
  when both are supplied. Empty input removes the corresponding old frames.

## Using The Feature

1. Enable experimental embedding in LRCGET's settings.
2. Select the embedded export preference in the existing export settings/menu
   if you want it selected initially in the editor.
3. Open the track's lyrics editor and make your edits.
4. Open the Save menu, select the desired formats and click **Save and export**.

Selecting a checkbox does not export or publish anything. **Save and Publish**
remains a separate, explicit action. Installation alone does not modify tracks.

## Format And Preservation

The existing writer uses ID3v2.4 with UTF-8 for MP3 USLT (plain text) and SYLT
(synchronized text) frames. SYLT uses milliseconds from the beginning of the
audio file and content type Lyrics. This change preserves supplied lyric text
and timestamps; it does not lowercase, transcribe, realign or invent lyrics.

Artwork and unrelated metadata remain outside the lyric-frame replacement.
The existing MP3 writer's comment handling is unchanged. Audio is not
re-encoded. FLAC export behavior is unchanged.

Valid tags do not guarantee lyric display in every player. Older stereos or
players may not support ID3v2.4 or synchronized lyrics. This contribution does
not add player compatibility, independent singing-timing verification, or a
word-by-word alignment engine.

## Tests

Frontend tests cover initial LRC selection, saved embedding selection, disabled
embedding, reopening the menu and preservation of user checkbox changes.
Rust tests cover replacement of multiple language/description variants, empty
lyrics, repeated exports, tag version/encoding, millisecond timestamps, artwork,
title and unchanged encoded audio packets in a generated MP3 fixture.
No user audio is included and no new test/runtime dependency is introduced.

```bash
npm test
npm run lint
npm run build
cd src-tauri
cargo check --locked
cargo test --locked
```

These changes intentionally leave upstream release numbering, database defaults
and the experimental opt-in unchanged.
