# batrust - Logitech Battery Tray Indicator

A cross-platform system tray application to monitor Logitech device battery levels, written in Rust.

## Features

- Monitors Logitech device battery levels via HID++ protocol
- Displays battery percentage in system tray
- Shows different icons based on battery level and charging status
- Configurable update intervals and color thresholds
- Minimal resource usage (<5MB RAM, 0% CPU when idle)

## Configuration

Configuration is stored in the user's config directory:
- Windows: `%USERPROFILE%\.config\batrust\config.toml`
- Linux: `~/.config/batrust/config.toml`
- macOS: `~/.config/batrust/config.toml`

The configuration file is automatically created with default values on first run:

```toml
update_interval = 60          # Update interval in seconds
red_threshold = 20            # Below this value, show as red
yellow_threshold = 30         # Below this value, show as yellow
disable_red = false           # Whether to disable red color
disable_yellow = false        # Whether to disable yellow color
```

## Building

To build the project:

```bash
cargo build --release
```

## Running

To run the project:

```bash
cargo run
```

## Architecture

The application consists of several modules:

- `hidpp`: Handles communication with Logitech devices using the HID++ protocol
- `tray`: Platform-specific system tray implementations
- `config`: Configuration loading and management

## License

MIT