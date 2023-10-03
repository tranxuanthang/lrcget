# LRCGET

Utility for mass-downloading LRC synced lyrics for your offline music library.

LRCGET will scan every files in your chosen directory for music files, then and try to download lyrics to a LRC files having the same name and save them to the same directory as your music files.

LRCGET is the official client of [LRCLIB](https://lrclib.net) service.

## Screenshots

![01.png](screenshots/01.png?2)

![02.png](screenshots/02.png?2)

![03.png](screenshots/03.png?2)

![04.png](screenshots/04.png?2)

![05.png](screenshots/04.png?2)

## Download

🎉 Newest version: v0.2.0

Visit the [release page](https://github.com/tranxuanthang/lrcget/releases).

OS Support:

- [x] Windows 10
- [x] Linux
- [x] macOS (experimental)

## Donation

Toss a coin to your developer?

Buy Me a Coffee:

<a href="https://www.buymeacoffee.com/thangtran" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>

Monero (XMR):

```
43ZN5qDdGQhPGthFnngD8rjCHYLsEFBcyJjDC1GPZzVxWSfT8R48QCLNGyy6Z9LvatF5j8kSgv23DgJpixJg8bnmMnKm3b7
```

## TODO

- [x] Choose multiple directories
- [x] Support .ogg format
- [ ] Embedded lyrics option ([temporary alternative](https://github.com/TheRedSpy15/lrcput))

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
