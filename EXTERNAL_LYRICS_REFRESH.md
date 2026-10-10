# Matching External Lyrics Refresh

After another application creates `Song.lrc` beside `Song.mp3`, use the existing
library Refresh action. A valid LRC containing timed words upgrades Plain or
missing lyrics, even when the recording is unchanged. Its words take precedence
over `Song.txt`. If no valid LRC exists, matching TXT fills missing lyrics.

Already synced lyrics and instrumental records stay protected. Explicitly
import a changed LRC to replace an already synced working document. Refresh
never exports, embeds, rewrites or deletes music/sidecar files.

The parser's existing timestamp formats remain unchanged. Untimed, metadata-only
and empty-timestamp LRCs are rejected as synced sources. The existing instrumental
marker is supported. Imports share the scan's batch transaction.

Regression tests cover database reopening, unchanged audio, TXT fallback,
invalid LRCs, LRC preference, protected saved synced edits and unchanged source
bytes with both fingerprint methods. Fixture audio is generated silence; no
personal recordings are included. No upstream release-version change is needed.
