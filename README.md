# NetBoard

Ultra-fast, ultra-lightweight network keyboard and mouse sharing application. Share your keyboard and mouse across multiple computers over the network with minimal latency.

## Features

- **Ultra-Fast**: Built with Rust for maximum performance and minimal overhead
- **Lightweight**: Binary protocol over UDP for lowest possible latency
- **Cross-Platform**: Works on Windows, Linux, and macOS
- **Simple**: Two-mode design - client sends input, server receives and injects it
- **No Dependencies**: Single binary, no installation required

## Technology Stack

- **Language**: Rust (zero-overhead abstractions, memory-safe)
- **Protocol**: UDP with custom binary serialization
- **Input Handling**: rdev library for cross-platform input capture/injection
- **Binary Size**: ~3-5MB (release build with optimizations)

## Installation

### From Source

```bash
cargo build --release
```

The compiled binary will be at `target/release/netboard`

### Quick Install

```bash
cargo install --path .
```

## Usage

### Server Mode (Target Computer)

Run this on the computer where you want to inject the keyboard/mouse input:

```bash
# Listen on all interfaces, port 9999 (default)
netboard server

# Or specify a custom bind address
netboard server --bind 0.0.0.0:8888
```

### Client Mode (Source Computer)

Run this on the computer whose keyboard/mouse you want to share:

```bash
# Connect to server at 192.168.1.100 on port 9999
netboard client --server 192.168.1.100:9999
```

## Example Scenarios

### Scenario 1: Control your Linux laptop from Windows desktop

**On Linux laptop (192.168.1.50):**
```bash
./netboard server --bind 0.0.0.0:9999
```

**On Windows desktop:**
```bash
netboard.exe client --server 192.168.1.50:9999
```

Now your Windows keyboard and mouse will control the Linux laptop!

### Scenario 2: Multiple computers on local network

**On target computer:**
```bash
netboard server
```

**On source computer:**
```bash
# Replace with actual IP of target
netboard client --server 192.168.1.100:9999
```

## Performance

- **Latency**: Sub-millisecond on local network
- **Bandwidth**: ~1-5 KB/s for typical usage
- **CPU Usage**: <1% on both client and server
- **Memory**: ~2-5MB RAM

## Security Considerations

**Important**: This application sends input events over UDP without encryption. Only use on trusted networks!

For secure usage:
- Use only on local networks or VPNs
- Consider using a firewall to restrict access
- Future versions may include encryption

## Troubleshooting

### Permission Issues (Linux)

On Linux, you may need elevated permissions to capture/inject input:

```bash
sudo ./netboard server
sudo ./netboard client --server IP:PORT
```

### Firewall Blocking

Make sure port 9999 (or your custom port) is open:

**Linux (ufw):**
```bash
sudo ufw allow 9999/udp
```

**Windows:**
```powershell
New-NetFirewallRule -DisplayName "NetBoard" -Direction Inbound -Protocol UDP -LocalPort 9999 -Action Allow
```

### Client not connecting

- Verify server is running and listening
- Check IP address and port are correct
- Verify firewall allows UDP traffic
- Ensure both computers are on the same network (or routable)

## Architecture

```
┌─────────────────┐         UDP Packets        ┌─────────────────┐
│  Client         │   ──────────────────────>   │  Server         │
│                 │                              │                 │
│  - Captures     │   Keyboard/Mouse Events     │  - Receives     │
│    keyboard     │   (Binary Serialized)       │    events       │
│  - Captures     │                              │  - Injects      │
│    mouse        │                              │    keyboard     │
│  - Serializes   │                              │  - Injects      │
│  - Sends UDP    │                              │    mouse        │
└─────────────────┘                              └─────────────────┘
```

## Building for Production

The release build is already optimized for size and speed:

```bash
cargo build --release
```

Optimizations in `Cargo.toml`:
- LTO (Link Time Optimization)
- Optimized for speed (opt-level = 3)
- Single codegen unit
- Stripped symbols

## Command Line Options

```bash
# Client mode
netboard client --server <IP:PORT>

# Server mode
netboard server [--bind <IP:PORT>]  # default: 0.0.0.0:9999

# Help
netboard --help
netboard client --help
netboard server --help
```

## Limitations

- No encryption (use only on trusted networks)
- UDP protocol means no guaranteed delivery (typically not an issue on LAN)
- Mouse coordinates are absolute (not relative) - works best with same resolution displays

## Future Enhancements

Potential features for future versions:
- [ ] Clipboard sharing
- [ ] TLS/encryption support
- [ ] Relative mouse movement mode
- [ ] Multi-client support
- [ ] Hotkey to enable/disable sharing
- [ ] Configuration file support
- [ ] GUI version

## License

MIT License - feel free to use and modify as needed.

## Contributing

Contributions welcome! This is designed to be simple and fast - please keep that philosophy in mind for any changes.
