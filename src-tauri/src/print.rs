//! Printing, for a web view that will not print by itself.
//!
//! The web view on macOS ignores `window.print()`: it prints only when the
//! window around it asks. So the application, when it runs inside the shell,
//! prepares the page for paper exactly as it would in a browser and then
//! navigates to `wf-desktop://print`. The main window's navigation guard
//! recognises that address, cancels it, and opens the operating system's print
//! dialog for the web view. When printing is over the shell runs [`DONE`] in
//! the page, and the page puts itself back the way it was.
//!
//! That is two short sentences of script, both written here, both calling only
//! what the page chose to define. Nothing travels the other way: the page still
//! gets no command and no capability. The one request it can make is "print
//! me", by an address nobody outside the shell can answer; an older shell fails
//! to load it and the page stays where it is.
//!
//! How each platform knows printing is over:
//! - **macOS**: the print operation's own completion callback, which AppKit
//!   sends once the sheet is closed and the job has been handed to the printer
//!   (or cancelled).
//! - **Windows and Linux**: their web views do print `window.print()`, so the
//!   shell calls it in the page and waits for the page's own `afterprint`
//!   event, which both fire once their print dialog closes.
//!
//! The page keeps its own fallback (the next click or focus after two seconds,
//! or sixty seconds), so a platform that never reports back leaves the screen
//! wrong for a moment, not for good.

use tauri::{Url, WebviewWindow};

/// The scheme of every address meant for the shell rather than for a server.
pub const SCHEME: &str = "wf-desktop";

/// Run in the page once printing is over, whichever way it ended. A macro so
/// that the Windows and Linux script below can carry the same sentence.
macro_rules! done_script {
    () => {
        "window.__wfPrintDone && window.__wfPrintDone()"
    };
}

#[cfg(target_os = "macos")]
const DONE: &str = done_script!();

/// File > Print. The page gets to prepare itself first (open every section,
/// scale the calculation to one sheet), then asks for the dialog the same way
/// its own print button does. A page with nothing to prepare asks directly:
/// plain `window.print()` would do nothing at all on macOS.
pub const FROM_MENU: &str =
    "window.printCalcOnePage ? window.printCalcOnePage() : (location.href = 'wf-desktop://print')";

/// Windows and Linux: their web view prints on `window.print()` and fires
/// `afterprint` when its dialog closes, printed or not. The flag keeps the page
/// from being told twice.
#[cfg(any(not(target_os = "macos"), test))]
const PRINT_IN_PAGE: &str = concat!(
    "(function () {",
    "var told = false;",
    "function done() {",
    "if (told) return;",
    "told = true;",
    "window.removeEventListener('afterprint', done);",
    done_script!(),
    ";",
    "}",
    "window.addEventListener('afterprint', done);",
    "window.print();",
    "})()"
);

/// What an address meant for the shell asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// `wf-desktop://print`: open the print dialog for this page.
    Print,
    /// Our scheme, but nothing this version knows. Cancelled and ignored: a
    /// newer application talking to an older shell must not end up in the
    /// browser, or on the web view's own error page.
    Unknown,
}

/// `None` for every address that is not meant for the shell, which is every
/// address that should be treated as a navigation.
pub fn request(url: &Url) -> Option<Request> {
    // `Url` lowercases the scheme; the host of a scheme it does not know is kept
    // as written, so the comparison there is ours to make case-blind.
    if url.scheme() != SCHEME {
        return None;
    }
    let host = url.host_str().unwrap_or_default();
    let path = url.path();
    if host.eq_ignore_ascii_case("print") && (path.is_empty() || path == "/") {
        Some(Request::Print)
    } else {
        Some(Request::Unknown)
    }
}

/// Opens the print dialog for `window`, and tells the page when it is over.
///
/// Never runs the dialog on the caller's thread. This is called from inside the
/// web view's navigation callback, which is on the main thread, and there Tauri
/// would carry out a request to the web view immediately rather than queue it:
/// the dialog would open in the middle of a navigation decision, and on Linux
/// it would spin its own event loop inside it. A task hands the request to the
/// event loop instead, and it runs once the callback has returned.
pub fn print(window: WebviewWindow) {
    tauri::async_runtime::spawn(async move { start(&window) });
}

/// Tells the page printing is over. Also queued rather than run in place, for
/// the same reason: the callback that knows it is over runs inside AppKit.
#[cfg(target_os = "macos")]
fn finish(window: WebviewWindow) {
    log::info!("print dialog closed");
    tauri::async_runtime::spawn(async move {
        if let Err(error) = window.eval(DONE) {
            log::warn!("could not tell the page printing is over: {error}");
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn start(window: &WebviewWindow) {
    if let Err(error) = window.eval(PRINT_IN_PAGE) {
        log::warn!("could not open the print dialog: {error}");
    }
}

#[cfg(target_os = "macos")]
fn start(window: &WebviewWindow) {
    let done = window.clone();
    let result = window.with_webview(move |webview| {
        // SAFETY: Tauri hands over its own live WKWebView, on the main thread.
        unsafe { mac::run(webview.inner(), Box::new(move || finish(done))) }
    });
    if let Err(error) = result {
        log::warn!("could not open the print dialog: {error}");
    }
}

#[cfg(target_os = "macos")]
mod mac {
    //! `WKWebView.printOperation(with:)` with a completion callback.
    //!
    //! Tauri's own `print()` runs the same operation with no delegate, which
    //! gives no way to know when the sheet has closed. This is that call with a
    //! delegate whose one method is the callback.

    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::sync::Mutex;

    use objc2::rc::Retained;
    use objc2::runtime::{Bool, NSObject};
    use objc2::{define_class, msg_send, sel, AllocAnyThread, DefinedClass, MainThreadMarker};
    use objc2_app_kit::{NSPrintInfo, NSPrintOperation};
    use objc2_web_kit::WKWebView;

    pub struct Ivars {
        /// Taken the first time the callback runs. A mutex rather than a cell
        /// because AppKit may finish a print on a thread of its own.
        done: Mutex<Option<Box<dyn FnOnce() + Send>>>,
    }

    define_class!(
        // SAFETY: NSObject has no subclassing requirements and this does not
        // implement Drop.
        #[unsafe(super(NSObject))]
        #[name = "WaterFormPrintDelegate"]
        #[ivars = Ivars]
        pub struct Delegate;

        impl Delegate {
            #[unsafe(method(printOperationDidRun:success:contextInfo:))]
            fn did_run(&self, _operation: &NSPrintOperation, _success: Bool, _context: *mut c_void) {
                let done = self.ivars().done.lock().ok().and_then(|mut d| d.take());
                if let Some(done) = done {
                    done();
                }
            }
        }
    );

    impl Delegate {
        fn new(done: Box<dyn FnOnce() + Send>) -> Retained<Self> {
            let this = Self::alloc().set_ivars(Ivars {
                done: Mutex::new(Some(done)),
            });
            // SAFETY: NSObject's init, on a freshly allocated object.
            unsafe { msg_send![super(this), init] }
        }
    }

    thread_local! {
        /// AppKit does not keep the delegate alive, so this does, until the
        /// next print replaces it. One at a time: a second request while the
        /// sheet is up is turned away below.
        static CURRENT: RefCell<Option<Retained<Delegate>>> = const { RefCell::new(None) };
    }

    /// # Safety
    /// `webview` must be a live `WKWebView`.
    pub unsafe fn run(webview: *mut c_void, done: Box<dyn FnOnce() + Send>) {
        if MainThreadMarker::new().is_none() || webview.is_null() {
            log::warn!("print asked for off the main thread; ignored");
            done();
            return;
        }
        let webview = &*(webview as *const WKWebView);
        let Some(window) = webview.window() else {
            done();
            return;
        };
        // Already printing, or another sheet is up: a second sheet cannot be
        // attached, and the page is told at once so it does not stay prepared.
        if window.attachedSheet().is_some() {
            done();
            return;
        }

        // No margins of our own, as in Tauri's print: the page's print style
        // decides, the way it does in a browser.
        let info = NSPrintInfo::sharedPrintInfo();
        info.setTopMargin(0.0);
        info.setRightMargin(0.0);
        info.setBottomMargin(0.0);
        info.setLeftMargin(0.0);

        let operation = webview.printOperationWithPrintInfo(&info);
        // Without a frame the printing view has nothing to lay out, and some
        // versions of macOS print blank pages.
        if let Some(view) = operation.view() {
            view.setFrame(webview.bounds());
        }
        // The sheet leaves the window responsive while it is up.
        operation.setCanSpawnSeparateThread(true);

        let delegate = Delegate::new(done);
        operation.runOperationModalForWindow_delegate_didRunSelector_contextInfo(
            &window,
            Some(&delegate),
            Some(sel!(printOperationDidRun:success:contextInfo:)),
            std::ptr::null_mut(),
        );
        CURRENT.with(|current| *current.borrow_mut() = Some(delegate));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn the_print_request_is_recognised() {
        assert_eq!(request(&url("wf-desktop://print")), Some(Request::Print));
        // However the web view writes it back.
        assert_eq!(request(&url("wf-desktop://print/")), Some(Request::Print));
        assert_eq!(request(&url("WF-DESKTOP://PRINT")), Some(Request::Print));
        assert_eq!(
            request(&url("wf-desktop://print?from=calc#top")),
            Some(Request::Print)
        );
    }

    #[test]
    fn our_scheme_with_anything_else_is_ours_but_does_nothing() {
        assert_eq!(request(&url("wf-desktop://save")), Some(Request::Unknown));
        assert_eq!(
            request(&url("wf-desktop://print/all")),
            Some(Request::Unknown)
        );
        assert_eq!(request(&url("wf-desktop:print")), Some(Request::Unknown));
        assert_eq!(request(&url("wf-desktop://")), Some(Request::Unknown));
    }

    #[test]
    fn every_other_address_is_a_navigation() {
        for other in [
            "https://app.waterform.fabrikode.com/print",
            "https://print/",
            "http://localhost:3100/engineering",
            "tauri://localhost/connect.html",
            "blob:https://app.waterform.fabrikode.com/0a1b",
            "mailto:iletisim@fabrikode.com",
            "wf-desktopx://print",
            "desktop://print",
        ] {
            assert_eq!(request(&url(other)), None, "{other}");
        }
    }

    #[test]
    fn the_menu_prints_through_the_page_and_the_same_address() {
        // The page prepares itself when it can, and either way the dialog is
        // asked for by the address the navigation guard recognises.
        assert!(FROM_MENU.contains("window.printCalcOnePage()"));
        let address = FROM_MENU.split('\'').nth(1).unwrap();
        assert_eq!(request(&url(address)), Some(Request::Print));
    }

    #[test]
    fn every_platform_tells_the_page_the_same_way() {
        assert_eq!(
            done_script!(),
            "window.__wfPrintDone && window.__wfPrintDone()"
        );
        // Windows and Linux: told once, after the dialog, by the page's own event.
        assert!(PRINT_IN_PAGE.contains(done_script!()));
        let listens = PRINT_IN_PAGE.find("addEventListener('afterprint'").unwrap();
        let prints = PRINT_IN_PAGE.find("window.print()").unwrap();
        assert!(
            listens < prints,
            "listening has to start before the dialog opens"
        );
    }
}
