use std::cell::Cell;
use std::rc::Rc;

use ul_next::{app::App, platform, window::WindowFlags, Library};

fn main() {
    let lib = Library::linked();

    // known upstream quirk on Linux: Config::resource_path_prefix is not
    // reliably respected: https://github.com/ultralight-ux/Ultralight/issues/403
    // the working fix is to explicitly register the platform filesystem
    // rooted at "." and keep resources/ at its default location relative
    // to CWD (run from the directory that contains resources/).
    platform::enable_platform_filesystem(lib.clone(), ".").unwrap();

    let app = App::new(lib.clone(), None, None).unwrap();

    let window = app
        .create_window(
            1920,
            1080,
            false,
            WindowFlags {
                borderless: false,
                titled: true,
                resizable: true,
                maximizable: false,
                hidden: false,
            },
        )
        .unwrap();

    window.set_title("ultralight svelte spike");

    let overlay = Rc::new(
        window
            .create_overlay(window.width(), window.height(), 0, 0)
            .unwrap(),
    );

    // the overlay is a fixed-size quad independent of the window -- without
    // this, resizing the window (including a tiling WM forcibly resizing it
    // on creation, before any manual resize) leaves the overlay at its
    // original pixel size, which both letterboxes/stretches the rendered
    // page and desyncs where AppCore thinks screen coordinates land relative
    // to where the page's DOM elements actually are -- a very plausible
    // cause of "clicks don't register" as well as the visual glitching.
    {
        let overlay = overlay.clone();
        window.set_resize_callback(move |_window, width, height| {
            overlay.resize(width, height);
        });
    }

    // the registered platform filesystem is rooted at "." -- the real
    // Svelte build output (renderer/dist/) is symlinked into
    // target/release/svelte-app/ for this spike, same as the plain-HTML
    // spike symlinked its assets in.
    overlay
        .view()
        .load_url("file:///svelte-app/index.html")
        .unwrap();

    overlay.view().set_add_console_message_callback(|_view, _src, level, message, line, _col, source_id| {
        eprintln!("[console:{:?}] {}:{}: {}", level, source_id, line, message);
    });

    overlay.view().set_fail_loading_callback(|_view, _frame_id, is_main_frame, url, description, domain, code| {
        eprintln!("FAIL LOADING (main={}): {} -- {} ({} {})", is_main_frame, url, description, domain, code);
    });

    // One-shot injected setup after ~1s, once the app has actually mounted:
    // navigate to Library (so the real scrolling grid is visible on launch,
    // not just the Home carousel) and install the same rAF-driven
    // worst-frame-time reporter used in the plain-HTML spike and the other
    // two toolkits' spikes, logged via console.log so it comes back through
    // the Rust console callback. This is a page-side (rAF/paint-tied)
    // measurement, deliberately not the Rust-side App::set_update_callback,
    // which fires on AppCore's internal update loop rather than actual
    // painted frames and reads ~2ms/tick regardless of real paint cost --
    // not a comparable number.
    //
    // Real mouse input (click/hover) does work in this window -- it just
    // needs genuine incremental motion events to reach GLFW/AppCore, not a
    // single teleport-style warp (xdotool's default `mousemove` -- a series
    // of small `mousemove` calls works fine, and so does a real mouse).
    let frame = Cell::new(0u32);
    let did_setup = Cell::new(false);
    let overlay_for_update = overlay.clone();

    app.set_update_callback(move || {
        let overlay = &overlay_for_update;
        frame.set(frame.get() + 1);
        if !did_setup.get() && frame.get() > 60 {
            did_setup.set(true);
            let _ = overlay.view().evaluate_script(
                "[...document.querySelectorAll('button')].find(b => b.textContent.trim() === 'Library')?.click();\
                 window.__lastT=0; window.__worstDt=0; window.__winStart=0;\
                 function __tick(t){\
                   if(window.__lastT>0){const dt=t-window.__lastT; if(dt>window.__worstDt)window.__worstDt=dt;}\
                   window.__lastT=t;\
                   if(window.__winStart===0)window.__winStart=t;\
                   if(t-window.__winStart>2000){\
                     console.log('worst_frame_ms='+window.__worstDt.toFixed(2)+' over last 2s');\
                     window.__worstDt=0; window.__winStart=t;\
                   }\
                   requestAnimationFrame(__tick);\
                 }\
                 requestAnimationFrame(__tick);",
            );
        }
    });

    let app_rc = Rc::new(app);
    let app_close = app_rc.clone();
    window.set_close_callback(move |_window| {
        app_close.quit();
    });

    app_rc.run();
}
