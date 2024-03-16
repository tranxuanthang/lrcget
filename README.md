# LRCGET

Utility for mass-downloading LRC synced lyrics for your offline music library.

LRCGET will scan every files in your chosen directory for music files, then and try to download lyrics to a LRC files having the same name and save them to the same directory as your music files.

LRCGET is the official client of [LRCLIB](https://lrclib.net) service.

## Download

🎉 Newest version: v0.3.0

Visit the [release page](https://github.com/tranxuanthang/lrcget/releases) to download.

OS Support:

- [x] Windows 10
- [x] Linux (Ubuntu 22.04+ and AppImage build)
- [x] macOS

## Screenshots

![02.png](screenshots/02.png?2)

![03.png](screenshots/03.png?2)

![04.png](screenshots/04.png?2)

![05.png](screenshots/05.png?2)

## Donation

Toss a coin to your developer?

Buy Me a Coffee:

<a href="https://www.buymeacoffee.com/thangtran" target="_blank">
  <img
    src="https://img.buymeacoffee.com/button-api/?text=Buy me a coffee&emoji=&slug=thangtran&button_colour=FFDD00&font_colour=000000&font_family=Cookie&outline_colour=000000&coffee_colour=ffffff"
    width="235"
    height="72"
  />
</a>

Paypal:

https://paypal.me/tranxuanthang98

Monero (XMR):

```
43ZN5qDdGQhPGthFnngD8rjCHYLsEFBcyJjDC1GPZzVxWSfT8R48QCLNGyy6Z9LvatF5j8kSgv23DgJpixJg8bnmMnKm3b7
```

Litecoin (LTC):

```
ltc1q7texq5qsp59gclqlwf6asrqmhm98gruvz94a48
```

## Contact

If you prefer to contact by email:

[hoangtudevops@protonmail.com](mailto:hoangtudevops@protonmail.com)

## TODO

- [x] Choose multiple directories
- [x] Support .ogg format
- [ ] Embedded lyrics option ([temporary alternative](https://github.com/TheRedSpy15/lrcput))
- [x] Optimize performance for loading large music library (https://github.com/tranxuanthang/lrcget/issues/19)
- [ ] Search feature
- [ ] Volume control
- [ ] Mark song as instrumental (https://github.com/tranxuanthang/lrcget/issues/36)

## Development

LRCGET is made with [Tauri](https://tauri.app).

To start developing the application, you need to do the [prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) steps according to your operating system.

For example, you need the following components to start the development in Windows:

- Microsoft Visual Studio C++ Build Tools
- Rust 1.64.0 or higher
- NodeJS v16.18.0 or higher

Start the development window with the following command:

``` shell
cd lrcget
npm install
npm run tauri dev
```

## Building

Start the development window with the following command:

``` shell
cd lrcget
npm install
npm run tauri build
```

Your built binaries will be at the following application:

```
./src-tauri/target/release/
```

For more detailed instruction, follow the [building guide](https://tauri.app/v1/guides/building/) to build the application according to your OS platform.
