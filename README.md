# Irak Notes - Rust Version

A fast, feature-rich notepad application built with Rust and egui.

## Features

- **Auto-Save**: Automatically saves notes as you type
- **Line Numbers**: Visual line numbers with scroll sync
- **Modern UI**: Clean, modern interface with custom title bar
- **Font Customization**: Change font family (Alt+F) and size (Alt+S)
- **Color Picker**: Full HSV color picker for background (Ctrl+Alt+C)
- **Find Dialog**: Search with case-sensitive option (Ctrl+F)
- **Keyboard Shortcuts**: Press F1 to see all shortcuts
- **Fast Performance**: Native Rust speed

## Requirements

- Rust 1.70+ (install from https://rustup.rs/)
- Windows 10/11 (WGPU backend; no OpenGL requirement)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or run the executable directly:
```bash
./target/release/irak_notes
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N | New file |
| Ctrl+F | Find text |
| Alt+F | Change font |
| Alt+S | Change font size |
| Ctrl+MouseWheel | Zoom in/out |
| Ctrl+Alt+C | Change background color |
| Alt+Delete | Delete current file |
| Ctrl+R | Rename file |
| Ctrl+O | Open save folder |
| Alt+Q | Close application |
| Ctrl+Up | Go to first line |
| Ctrl+Down | Go to last line |
| F1 | Show shortcuts help |

## Auto-Save Location

Notes are automatically saved to: `~/Desktop/Irak Notes Auto Saved/`

## Project Structure

```
irak_notes_rust/
├── Cargo.toml           # Dependencies
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # Main application
│   ├── file_ops.rs      # File operations
│   ├── settings.rs      # Settings persistence
│   ├── line_numbers.rs  # Line numbers widget
│   ├── dialogs.rs       # Dialogs (Find, Font, etc.)
│   ├── color_picker.rs  # Color picker
│   └── shortcuts.rs     # Shortcuts help
└── README.md
```

## Dependencies

- `eframe` - Cross-platform GUI framework
- `egui` - Immediate mode GUI library
- `serde` - Serialization
- `dirs` - Cross-platform paths
- `open` - Open files in system
- `chrono` - Date/time

## License

MIT License

## Author

IroScript
