# Rust with raw Win32, no UI framework

Mai sits on screen all day, so the top priority is the lowest possible RAM and CPU while still working well. We write it in Rust against Win32 directly (the `windows` crate): a layered window drawn with `UpdateLayeredWindow` gives transparency, click-through and always-on-top in a few MB, with no runtime. The window is redrawn only when a frame changes, never in a continuous render loop. Windows-only is accepted.

## Considered Options

- C# WinForms/WPF: easier, but the .NET runtime costs roughly 20–60+ MB.
- Tauri / Electron: web rendering via WebView2/Chromium costs 50–200 MB across several processes.
- Godot: the engine keeps running in the background, which is heavy for a single sprite.
- C++ Win32: same footprint as Rust, but not memory-safe.
