# OpenGL Error Fix

If you see this error:
```
Error: OpenGL(PainterError("egui_glow requires opengl 2.0+. "))
```

## Solution: Use WGPU Backend

The project is already configured to use WGPU (DirectX/Vulkan) instead of OpenGL.

### Current Build Status

The project is building with WGPU support. This will take longer (5-10 minutes) but will work on systems without OpenGL 2.0+.

### After Build Completes

Run the executable:
```powershell
.\target\release\irak_notes.exe
```

### If Build is Still Running

Wait for the build to complete. You'll see:
```
Finished `release` profile [optimized] target(s) in X minutes
```

### Alternative: Use Python Version

If the Rust build takes too long, you can use the Python version:
```powershell
cd ..
python Irak_Note.py
```

The Python version has all the same features and works immediately.

## Technical Details

- **Old backend**: `glow` (OpenGL)
- **New backend**: `wgpu` (DirectX 12 on Windows, Vulkan on Linux, Metal on macOS)
- **Benefit**: Works on more systems, better performance
- **Drawback**: Larger binary size, longer compile time

## Cargo.toml Changes

```toml
[dependencies]
eframe = { version = "0.27", default-features = false, features = ["default_fonts", "wgpu"] }
```

This disables OpenGL and enables WGPU.
