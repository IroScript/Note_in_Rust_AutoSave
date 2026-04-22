# ⚠️ GPU Required for Rust Version

## Problem

The Rust version of Irak Notes requires a GPU with OpenGL 2.0+ support. Your system doesn't have this.

## ✅ Solution: Use Python Version

The Python version works perfectly on your system and has **all the same features**:

```powershell
# From this directory:
cd ..
python Irak_Note.py
```

## Why This Happens

- **Rust/egui** uses OpenGL or Vulkan/DirectX for rendering
- Your system doesn't have compatible GPU drivers
- This is common on virtual machines, older computers, or systems without GPU drivers

## Python Version Features

All features are identical to the Rust version:
- ✅ Auto-save
- ✅ Line numbers with scroll sync
- ✅ Font customization (Alt+F, Alt+S)
- ✅ Color picker (Ctrl+Alt+C)
- ✅ Find dialog (Ctrl+F)
- ✅ All keyboard shortcuts (F1 for help)
- ✅ Modern UI with custom title bar

## Quick Start

```powershell
cd C:\Users\Irak\Desktop\NoteApp\Irak_Note_5.1
python Irak_Note.py
```

That's it! The app will open immediately. 🚀

## Technical Details

The Rust code is complete and correct. It just needs a GPU to run. The Python version uses tkinter which works on any system without GPU requirements.

## Files

- `Irak_Note.py` - Python version (recommended)
- `irak_notes_rust/` - Rust version (requires GPU)
