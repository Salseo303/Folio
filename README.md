# Folio

A small, fast, offline EPUB reader for Windows, with an OLED black theme.

Folio started as a single HTML file and is now wrapped in a [Tauri v2](https://tauri.app)
desktop app. The whole interface is one file: `src/index.html`.

## Features

- Library of your EPUBs that remembers **where each book is** (it never copies your books)
- Remembers chosen folders, rescans them on launch, and has a **Refresh** button that adds
  new files and removes deleted ones
- Cover thumbnails in the library list (letter tile when a book has no usable cover)
- One bookmark per book, saved between launches
- Settings (text size, line spacing, width, font) saved to a JSON file
- Drag and drop EPUBs or folders onto the window
- "Open with Folio" for `.epub` files (from the installer), one window only
- F11 fullscreen
- Opens large (50 MB+) illustrated books quickly by reading only the parts of the file it needs
- Fully offline: no network requests, no telemetry, no accounts
- Book content is sanitised (no scripts, iframes, or remote resources) and the app runs under
  a strict Content Security Policy

## Where your data lives

Everything is in `%APPDATA%\com.folio.reader\`:

| File | Contents |
|---|---|
| `settings.json` | Reader settings |
| `library.json` | Book paths, titles, bookmarks, remembered folders |
| `covers.json` | Small cover thumbnails |

Your EPUB files are never modified or copied.

## Build it

Requirements (Windows 10/11): [Rust](https://rustup.rs) (stable), the Visual Studio
C++ Build Tools, and WebView2 (included with Windows 11 and current Edge). No Node.js is needed.

```powershell
cargo install tauri-cli --version "^2"

cargo tauri dev      # run the app
cargo tauri build    # release build and installer
```

The installer appears in `src-tauri\target\release\bundle\nsis\`. It is not code-signed, so
Windows SmartScreen may show a warning (More info, then Run anyway).

## Use the HTML in a browser

You can also open `src/index.html` directly in a browser. It still works, but without the
app-only features: the library lives in memory, there are no remembered folders, covers,
drag and drop, or Refresh, and settings use `localStorage`.

## Project layout

```
src/index.html            the whole reader (HTML, CSS and JS)
src-tauri/                the Tauri (Rust) wrapper: file access, settings, single instance
icon-source.png           source for the app icon
epub-reader.original.html the original single-file reader, kept for reference
```

## Credits

Folio was built with AI assistance:

- The original single-file reader (`epub-reader.original.html`) was written with Claude Sonnet 5.5
- The build specification and the prompts for the desktop app were also written with Claude.
- The desktop app code was written by DeepSeek V4.1 Flash, running in OpenCode

Design decisions and testing by me.

## License

[MIT](LICENSE)
