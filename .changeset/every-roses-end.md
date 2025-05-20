---
"lrcget": minor
---

Add comprehensive error handling and validation to various files.

cli/src/bitrate.ts

Add error handling for invalid bitrate input in parseBitrate function.
Add validation for bitrate input in getBitrateNumberFromText function.
cli/src/download.ts

Add error handling for invalid URLs in downloadLinks function.
Add validation for settings and Spotify plugin in downloadLinks function.
cli/src/login.ts

Add error handling for invalid ARL input in deezerLogin function.
Add validation for ARL file existence in deezerLogin function.
deemix/src/decryption.ts

Add error handling for decryption errors in streamTrack function.
Add validation for track download URL in streamTrack function.
deemix/src/download-objects/Collection.ts

Add error handling for invalid collection data in Collection class constructor.
Add validation for conversion data in Convertable class constructor.
deemix/src/download-objects/DownloadObject.ts

Add error handling for invalid download object data in DownloadObject class constructor.
Add validation for progress update in updateProgress function.
deemix/src/downloader.ts

Add error handling for invalid track data in download function.
Add validation for track download URL in downloadWrapper function.
