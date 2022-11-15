# LRCGET

Utility for mass-downloading LRC synced lyrics for your offline music library.

LRCGET will scan every files in your chosen directory for music files, then and try to download lyrics to a LRC files having the same name and save them to the same directory as your music files.

## Screenshots

![01.png](screenshots/01.png)

![02.png](screenshots/02.png)

## Download

Visit the [release page](https://github.com/tranxuanthang/lrcget/releases).

OS Support:

- [x] Windows 10
- [ ] Linux (coming soon)
- [ ] macOS (coming soon)

## TODO

- [ ] Embedded lyrics option
- [ ] Choose multiple directories

## Development

LRCGET is made with [Tauri](https://tauri.app).

To start developing the application, you need to do the [prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) steps according to your operating system.

For example, you need the following components to start the development in Windows:
- Microsoft Visual Studio C++ Build Tools
- Rust 1.64.0 or higher
- NodeJS v16.18.0 or higher

Start the development window with the following command:

``` shell
npm run tauri dev
```

Follow the [building guide](https://tauri.app/v1/guides/building/) to build the application according to your OS platform.
