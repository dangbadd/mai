//! Win32 shell: a thin adapter between Windows and the pure `mai` core. Not unit tested; it is
//! verified against the manual checklist.
#![windows_subsystem = "windows"]

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use mai::assets::{self, Animation};
use mai::sim::{Character, Clip, Draw, MouseEvent, World};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{HSTRING, w};

/// Character height at 100% display scale.
const BASE_SIZE: u32 = 128;
const WM_TRAY: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;
const CMD_QUIT: u32 = 1;
/// Broadcast by Explorer when the taskbar is recreated, which drops every tray icon.
static WM_TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

struct App {
    hwnd: HWND,
    character: Character,
    world: World,
    /// Premultiplied BGRA frames at display size, ready to copy into `bits`.
    frames: HashMap<String, Vec<Vec<u8>>>,
    size: i32,
    mem_dc: HDC,
    bits: *mut u8,
    last_draw: Option<Draw>,
    last_tick: Instant,
    /// Period the step timer currently runs at.
    interval: Duration,
    /// Mouse input received since the last step.
    events: Vec<MouseEvent>,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn main() {
    unsafe {
        // Must come before any window exists so sizes are in physical pixels.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

        // Held until the process exits; the OS releases it then.
        let _mutex = CreateMutexW(None, true, w!("Local\\Mai.SingleInstance"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }

        let dir = std::env::current_exe().unwrap().with_file_name("assets");
        let animations = match assets::load(&dir) {
            Ok(a) => a,
            Err(e) => {
                let text = HSTRING::from(format!("{}\n\n{}", e.file.display(), e.reason));
                MessageBoxW(None, &text, w!("Mai: asset error"), MB_OK | MB_ICONERROR);
                return;
            }
        };

        if let Err(e) = run(animations) {
            MessageBoxW(
                None,
                &HSTRING::from(e.message()),
                w!("Mai"),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

unsafe fn run(animations: HashMap<String, Animation>) -> windows::core::Result<()> {
    unsafe {
        let size = scaled_size()?;
        let world = primary_world(size)?;

        let instance = GetModuleHandleW(None)?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            lpszClassName: w!("Mai"),
            ..Default::default()
        };
        RegisterClassW(&class);
        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("Mai"),
            w!("Mai"),
            WS_POPUP,
            0,
            0,
            size,
            size,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let (mem_dc, bits) = create_canvas(size)?;
        let clips: HashMap<String, Clip> = animations
            .iter()
            .map(|(n, a)| (n.clone(), a.clip))
            .collect();
        // Consumes the source strips so only the display-size frames stay in memory.
        let frames = animations
            .into_iter()
            .map(|(name, a)| (name, scale_frames(&a, size as u32)))
            .collect();
        let character = Character::new(clips, &world);

        APP.with_borrow_mut(|app| {
            *app = Some(App {
                hwnd,
                character,
                world,
                frames,
                size,
                mem_dc,
                bits,
                last_draw: None,
                last_tick: Instant::now(),
                interval: Duration::ZERO,
                events: Vec::new(),
            })
        });
        tick();
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        WM_TASKBAR_CREATED.store(
            RegisterWindowMessageW(w!("TaskbarCreated")),
            Ordering::Relaxed,
        );
        add_tray_icon(hwnd)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}

/// Ground and screen edges from the primary monitor's work area, whose bottom is the taskbar's top
/// edge when the taskbar sits at the bottom. Edges are inset by half the Character so it stays
/// fully on screen.
unsafe fn primary_world(size: i32) -> windows::core::Result<World> {
    let mut area = RECT::default();
    unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut area as *mut RECT as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )?;
    }
    Ok(World {
        ground_y: area.bottom,
        left: area.left + size / 2,
        right: area.right - size / 2,
    })
}

unsafe fn scaled_size() -> windows::core::Result<i32> {
    let (mut dpi_x, mut dpi_y) = (0, 0);
    unsafe {
        let monitor = MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY);
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y)?;
    }
    Ok((BASE_SIZE * dpi_y / 96) as i32)
}

/// A memory DC with a top-down 32-bit DIB selected in, which `UpdateLayeredWindow` reads from.
unsafe fn create_canvas(size: i32) -> windows::core::Result<(HDC, *mut u8)> {
    unsafe {
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let dc = CreateCompatibleDC(None);
        SelectObject(dc, bitmap.into());
        Ok((dc, bits as *mut u8))
    }
}

/// Splits a straight-RGBA strip into frames and box-filters each to `size`², premultiplied BGRA.
fn scale_frames(a: &Animation, size: u32) -> Vec<Vec<u8>> {
    let (src, stride) = (a.frame_size, a.frame_size * a.clip.frames);
    (0..a.clip.frames)
        .map(|f| {
            let mut out = Vec::with_capacity((size * size * 4) as usize);
            for dy in 0..size {
                let (y0, y1) = span(dy, size, src);
                for dx in 0..size {
                    let (x0, x1) = span(dx, size, src);
                    let mut sum = [0u32; 4];
                    for y in y0..y1 {
                        for x in x0..x1 {
                            let i = ((y * stride + f * src + x) * 4) as usize;
                            let p = &a.rgba[i..i + 4];
                            let alpha = p[3] as u32;
                            sum[0] += p[2] as u32 * alpha / 255;
                            sum[1] += p[1] as u32 * alpha / 255;
                            sum[2] += p[0] as u32 * alpha / 255;
                            sum[3] += alpha;
                        }
                    }
                    let n = (y1 - y0) * (x1 - x0);
                    out.extend(sum.map(|c| (c / n) as u8));
                }
            }
            out
        })
        .collect()
}

/// Source pixels `[start, end)` covering destination pixel `d`; at least one wide when upscaling.
fn span(d: u32, dst: u32, src: u32) -> (u32, u32) {
    let start = d * src / dst;
    (start, ((d + 1) * src / dst).max(start + 1))
}

unsafe fn add_tray_icon(hwnd: HWND) -> windows::core::Result<()> {
    unsafe {
        let mut data = tray_data(hwnd);
        data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        data.uCallbackMessage = WM_TRAY;
        data.hIcon = LoadIconW(None, IDI_APPLICATION)?;
        let tip: Vec<u16> = "Mai".encode_utf16().collect();
        data.szTip[..tip.len()].copy_from_slice(&tip);
        Shell_NotifyIconW(NIM_ADD, &data).ok()
    }
}

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    }
}

/// Steps the simulation by the real time since the last tick, redraws only if the frame or position
/// changed, and re-arms the timer when the step rate changes (fast while walking, slow otherwise).
fn tick() {
    APP.with_borrow_mut(|app| {
        let Some(app) = app else { return };
        let now = Instant::now();
        let draw = app
            .character
            .step(&app.events, now - app.last_tick, &app.world, &mut rand_u32);
        app.events.clear();
        app.last_tick = now;
        if app.last_draw != Some(draw) {
            app.last_draw = Some(draw);
            unsafe { present(app, draw) };
        }
        let interval = app.character.next_step_in();
        if interval != app.interval {
            app.interval = interval;
            unsafe { SetTimer(Some(app.hwnd), TIMER_ID, interval.as_millis() as u32, None) };
        }
    });
}

unsafe fn present(app: &App, draw: Draw) {
    let frame = &app.frames[draw.animation][draw.frame as usize];
    unsafe {
        let bits = std::slice::from_raw_parts_mut(app.bits, frame.len());
        if draw.mirrored {
            let row = app.size as usize * 4;
            for (dst, src) in bits.chunks_mut(row).zip(frame.chunks(row)) {
                for (d, s) in dst.chunks_mut(4).zip(src.chunks(4).rev()) {
                    d.copy_from_slice(s);
                }
            }
        } else {
            bits.copy_from_slice(frame);
        }
        let pos = POINT {
            x: draw.x - app.size / 2,
            y: draw.y - app.size,
        };
        let size = SIZE {
            cx: app.size,
            cy: app.size,
        };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
            ..Default::default()
        };
        let _ = UpdateLayeredWindow(
            app.hwnd,
            None,
            Some(&pos),
            Some(&size),
            Some(app.mem_dc),
            Some(&POINT::default()),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }
}

/// Queues a mouse event at the cursor's screen position when the message was sent. Windows only
/// sends these for pixels the Character covers: fully transparent ones fall through.
fn push_mouse(event: fn(i32, i32) -> MouseEvent) {
    let pos = unsafe { GetMessagePos() };
    let event = event(pos as i16 as i32, (pos >> 16) as i16 as i32);
    APP.with_borrow_mut(|app| {
        if let Some(app) = app {
            app.events.push(event);
        }
    });
}

/// Each `RandomState` gets fresh SipHash keys, so hashing a constant yields a new random value.
fn rand_u32() -> u32 {
    use std::hash::BuildHasher;
    std::hash::RandomState::new().hash_one(0u8) as u32
}

unsafe fn show_tray_menu(hwnd: HWND) {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else { return };
        let _ = AppendMenuW(menu, MF_STRING, CMD_QUIT as usize, w!("Quit"));
        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        // Without this the menu does not close when the user clicks elsewhere.
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_NONOTIFY,
            cursor.x,
            cursor.y,
            None,
            hwnd,
            None,
        );
        // Documented workaround so the next tray click opens the menu instead of only closing it.
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        if cmd.0 as u32 == CMD_QUIT {
            let _ = Shell_NotifyIconW(NIM_DELETE, &tray_data(hwnd));
            let _ = DestroyWindow(hwnd);
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_TIMER if wparam.0 == TIMER_ID => tick(),
            WM_LBUTTONDOWN => push_mouse(|x, y| MouseEvent::Press { x, y }),
            WM_MOUSEMOVE => push_mouse(|x, y| MouseEvent::Move { x, y }),
            WM_LBUTTONUP => {
                push_mouse(|x, y| MouseEvent::Release { x, y });
                // React now rather than on the next timer, which may be up to a second away.
                tick();
            }
            WM_TRAY if matches!(lparam.0 as u32, WM_RBUTTONUP | WM_CONTEXTMENU) => {
                show_tray_menu(hwnd)
            }
            WM_DESTROY => PostQuitMessage(0),
            _ if msg != 0 && msg == WM_TASKBAR_CREATED.load(Ordering::Relaxed) => {
                let _ = add_tray_icon(hwnd);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
        LRESULT(0)
    }
}
