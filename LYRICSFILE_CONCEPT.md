```yaml
version: "1.0"

metadata:
  title: "Song Title"
  artist: "Artist Name"
  album: "Album Name"
  duration_ms: 245000
  offset_ms: 0
  language: "en"
  instrumental: false

lines:
  - text: "Synced line here"
    start_ms: 12000
    end_ms: 15500

plain: |
  Song Title

  [Verse 1]
  Synced line here
  Another synced line

  [Chorus]
  Hook line here
```

---

## Specification

### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | string | Yes | Format version, always `"1.0"` |
| `metadata` | object | Yes | Song information |
| `lines` | array | No | Synced lyric lines (omit if unsynced) |
| `plain` | string | No | Plain text lyrics (multiline) |

### Metadata Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Song title |
| `artist` | string | Yes | Primary artist |
| `album` | string | No | Album name |
| `duration_ms` | integer | No | Total song length in milliseconds |
| `offset_ms` | integer | No | Global timing offset in milliseconds (default: `0`) |
| `language` | string | No | ISO 639-1 language code |
| `instrumental` | boolean | No | `true` if song has no vocals (default: `false`) |

### Line Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | string | Yes | Full line text |
| `start_ms` | integer | Yes | Line start time in milliseconds |
| `end_ms` | integer | No | Line end time in milliseconds |
| `words` | array | No | Word-level sync array |

### Word Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text` | string | Yes | Word text including trailing space (except last word) |
| `start_ms` | integer | Yes | Word start time in milliseconds |
| `end_ms` | integer | No | Word end time in milliseconds |

---

## Usage Patterns

### 1. Unsynced Lyrics Only
```yaml
version: "1.0"
metadata:
  title: "New Song"
  artist: "Unknown Artist"
  instrumental: false

lines: []

plain: |
  [Verse 1]
  These lyrics haven't been synced yet

  Just plain text with empty lines for spacing

  [Chorus]
  La la la
```

### 2. Synced + Plain (Customized)
```yaml
version: "1.0"
metadata:
  title: "Midnight City"
  artist: "M83"
  duration_ms: 243000

lines:
  - text: "Waiting in a car"
    start_ms: 23500
    end_ms: 26800
  - text: "Waiting for a ride in the dark"
    start_ms: 26800
    end_ms: 31500

plain: |
  MIDNIGHT CITY

  [Verse 1]
  Waiting in a car
  Waiting for a ride in the dark

  (Instrumental break)

  [Verse 2]
  The city is my church
```
*Note: Plain version includes custom headers, empty lines, and notes that wouldn't exist in the synced array.*

### 3. Instrumental (No Lyrics)
```yaml
version: "1.0"
metadata:
  title: "Adagio in G Minor"
  artist: "Tomaso Albinoni"
  duration_ms: 480000
  instrumental: true

lines: []
plain: ""  # Empty or omitted
```

---

## Rules

1. **Timing**: All timestamps are integers in milliseconds, monotonically increasing
2. **Words array**: If present, render words sequentially; ignore `line.text` for display
3. **Trailing spaces**: Include in `word.text` except for the last word of each line
4. **CJK**: No spaces needed between words
5. **Validation**: Concatenation of `word.text` should approximate `line.text`
6. **Instrumental**: When `true`, both `lines` and `plain` should be empty or omitted
7. **Plain field**: Uses literal block scalar (`|`) to preserve newlines and spacing exactly as written
