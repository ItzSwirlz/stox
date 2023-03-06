# Stox - GTK Application for everything stock market

Stox is a WIP application for viewing the stock market with Yahoo Finance.

![Screenshot](https://raw.githubusercontent.com/ItzSwirlz/stox/main/.github/screenshot.png)

## Building

You will need:
- The GTK 4 development library
- Rust compiler and Cargo
- Meson and ninja-build
- Gettext (for translations)

You will also need to build and install so the gresource and schemas files can be found during runtime.
```
meson build
ninja -C build
sudo ninja -C build install
```

## Contributing
Just send a PR! For translations, go [here](https://github.com/users/ItzSwirlz/projects/4/views/1).
