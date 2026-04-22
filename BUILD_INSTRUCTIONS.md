# Quick Build Instructions

## SSL Certificate Fix (Required for Windows)

Your system has SSL certificate issues. Here are solutions:

### Solution 1: Environment Variable (Quickest)

```powershell
$env:CARGO_HTTP_CHECK_REVOKE="false"
cargo build --release
```

### Solution 2: Update Windows Certificates

1. Open Windows Update
2. Install all pending updates
3. Restart your computer
4. Try building again

### Solution 3: Use Git Protocol

Edit `Cargo.toml` and add at the top:

```toml
[patch.crates-io]
# Use git protocol instead of HTTPS
```

## After Fixing SSL

```bash
# Build
cargo build --release

# Run
cargo run --release

# Or run the executable
./target/release/irak_notes.exe
```

## Alternative: Use Cargo Vendor

If SSL issues persist:

```bash
# Download all dependencies once (on a working machine)
cargo vendor

# Then build offline
cargo build --release --offline
```

## Notes

- The `.cargo/config.toml` file is already configured to help with SSL
- All Rust source files are in `src/` directory
- The project is a complete 1:1 port of the Python version
- All features, shortcuts, and behaviors are preserved
