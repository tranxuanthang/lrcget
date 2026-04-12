```yaml
version: '1.0'

metadata:
  title: 'Song Title'
  artist: 'Artist Name'
  album: 'Album Name'
  duration_ms: 245000
  offset_ms: 0
  language: 'en'
  instrumental: false

lines:
  - text: 'Synced line here'
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

| Field      | Type   | Required | Description                           |
| ---------- | ------ | -------- | ------------------------------------- |
| `version`  | string | Yes      | Format version, always `"1.0"`        |
| `metadata` | object | Yes      | Song information                      |
| `lines`    | array  | No       | Synced lyric lines (omit if unsynced) |
| `plain`    | string | No       | Plain text lyrics (multiline)         |

### Metadata Object

| Field          | Type    | Required | Description                                         |
| -------------- | ------- | -------- | --------------------------------------------------- |
| `title`        | string  | Yes      | Song title                                          |
| `artist`       | string  | Yes      | Primary artist                                      |
| `album`        | string  | No       | Album name                                          |
| `duration_ms`  | integer | No       | Total song length in milliseconds                   |
| `offset_ms`    | integer | No       | Global timing offset in milliseconds (default: `0`) |
| `language`     | string  | No       | ISO 639-1 language code                             |
| `instrumental` | boolean | No       | `true` if song has no vocals (default: `false`)     |

### Line Object

| Field      | Type    | Required | Description                     |
| ---------- | ------- | -------- | ------------------------------- |
| `text`     | string  | Yes      | Full line text                  |
| `start_ms` | integer | Yes      | Line start time in milliseconds |
| `end_ms`   | integer | No       | Line end time in milliseconds   |
| `words`    | array   | No       | Word-level sync array           |

### Word Object

| Field      | Type    | Required | Description                                           |
| ---------- | ------- | -------- | ----------------------------------------------------- |
| `text`     | string  | Yes      | Word text including trailing space (except last word) |
| `start_ms` | integer | Yes      | Word start time in milliseconds                       |
| `end_ms`   | integer | No       | Word end time in milliseconds                         |

---

## Usage Patterns

### 1. Unsynced Lyrics Only

```yaml
version: '1.0'
metadata:
  title: 'New Song'
  artist: 'Unknown Artist'
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
version: '1.0'
metadata:
  title: 'Midnight City'
  artist: 'M83'
  duration_ms: 243000

lines:
  - text: 'Waiting in a car'
    start_ms: 23500
    end_ms: 26800
  - text: 'Waiting for a ride in the dark'
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

_Note: Plain version includes custom headers, empty lines, and notes that wouldn't exist in the synced array._

### 3. Instrumental (No Lyrics)

```yaml
version: '1.0'
metadata:
  title: 'Adagio in G Minor'
  artist: 'Tomaso Albinoni'
  duration_ms: 480000
  instrumental: true

lines: []
plain: '' # Empty or omitted
```

### 4. Word-Synced Lyrics

```yaml
version: '1.0'
metadata:
  title: 'Shape of You'
  artist: 'Ed Sheeran'
  duration_ms: 235000

lines:
  - text: "The club isn't the best place to find a lover"
    start_ms: 12450
    end_ms: 18200
    words:
      - text: 'The '
        start_ms: 12450
        end_ms: 12900
      - text: 'club '
        start_ms: 12900
        end_ms: 13500
      - text: "isn't "
        start_ms: 13500
        end_ms: 14200
      - text: 'the '
        start_ms: 14200
        end_ms: 14600
      - text: 'best '
        start_ms: 14600
        end_ms: 15200
      - text: 'place '
        start_ms: 15200
        end_ms: 15800
      - text: 'to '
        start_ms: 15800
        end_ms: 16200
      - text: 'find '
        start_ms: 16200
        end_ms: 16800
      - text: 'a '
        start_ms: 16800
        end_ms: 17100
      - text: 'lover'
        start_ms: 17100
        end_ms: 18200
  - text: 'So the bar is where I go'
    start_ms: 18500
    end_ms: 22100
    words:
      - text: 'So '
        start_ms: 18500
        end_ms: 19000
      - text: 'the '
        start_ms: 19000
        end_ms: 19400
      - text: 'bar '
        start_ms: 19400
        end_ms: 20000
      - text: 'is '
        start_ms: 20000
        end_ms: 20400
      - text: 'where '
        start_ms: 20400
        end_ms: 21000
      - text: 'I '
        start_ms: 21000
        end_ms: 21400
      - text: 'go'
        start_ms: 21400
        end_ms: 22100

plain: |
  [Verse 1]
  The club isn't the best place to find a lover
  So the bar is where I go
```

_Note: Word objects include trailing spaces in `text` (except the last word) to allow proper reconstruction of the full line. Each word has its own `start_ms` for karaoke-style highlighting._

---

## Rules

1. **Timing**: All timestamps are integers in milliseconds, monotonically increasing
2. **Words array**: If present, render words sequentially; ignore `line.text` for display
3. **Trailing spaces**: Include in `word.text` except for the last word of each line
4. **CJK**: No spaces needed between words
5. **Validation**: Concatenation of `word.text` should approximate `line.text`
6. **Instrumental**: When `true`, both `lines` and `plain` should be empty or omitted
7. **Plain field**: Uses literal block scalar (`|`) to preserve newlines and spacing exactly as written
