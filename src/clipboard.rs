#[cfg(any(test, not(any(target_os = "linux", target_os = "macos", windows))))]
use copypasta::nop_clipboard::NopClipboardContext;
#[cfg(target_os = "linux")]
use copypasta::x11_clipboard::{Primary as X11SelectionClipboard, X11ClipboardContext};
#[cfg(any(target_os = "linux", target_os = "macos", windows))]
use copypasta::ClipboardContext;
use copypasta::ClipboardProvider;

pub struct Clipboard {
    clipboard: Box<dyn ClipboardProvider>,
    selection: Option<Box<dyn ClipboardProvider>>,
}

impl Clipboard {
    #[cfg(any(target_os = "linux", target_os = "macos", windows))]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(any(test, not(any(target_os = "linux", target_os = "macos", windows))))]
    pub fn new_nop() -> Self {
        Self {
            clipboard: Box::new(NopClipboardContext::new().unwrap()),
            selection: None,
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        #[cfg(any(target_os = "macos", windows))]
        return Self {
            clipboard: Box::new(ClipboardContext::new().unwrap()),
            selection: None,
        };

        #[cfg(target_os = "linux")]
        return Self {
            clipboard: Box::new(ClipboardContext::new().unwrap()),
            selection: Some(Box::new(
                X11ClipboardContext::<X11SelectionClipboard>::new().unwrap(),
            )),
        };

        #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
        return Self::new_nop();
    }
}

impl Clipboard {
    pub fn store(&mut self, text: impl Into<String>) {
        let clipboard = match &mut self.selection {
            Some(provider) => provider,
            None => &mut self.clipboard,
        };

        clipboard.set_contents(text.into()).unwrap_or_else(|err| {
            panic!("Unable to store text in clipboard: {}", err);
        });
    }

    pub fn _load(&mut self) -> String {
        let clipboard = match &mut self.selection {
            Some(provider) => provider,
            None => &mut self.clipboard,
        };

        match clipboard.get_contents() {
            Err(err) => {
                panic!("Unable to load text from clipboard: {}", err);
            }
            Ok(text) => text,
        }
    }
}
