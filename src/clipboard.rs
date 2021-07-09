#[cfg(any(test, not(any(feature = "x11", target_os = "macos", windows))))]
use copypasta::nop_clipboard::NopClipboardContext;
#[cfg(any(target_os = "macos", windows))]
use copypasta::ClipboardContext;
use copypasta::ClipboardProvider;

pub struct Clipboard {
    clipboard: Box<dyn ClipboardProvider>,
}

impl Clipboard {
    #[cfg(any(target_os = "macos", windows))]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(any(test, not(any(target_os = "macos", windows))))]
    pub fn new_nop() -> Self {
        Self {
            clipboard: Box::new(NopClipboardContext::new().unwrap()),
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        #[cfg(any(target_os = "macos", windows))]
        return Self {
            clipboard: Box::new(ClipboardContext::new().unwrap()),
        };

        #[cfg(not(any(target_os = "macos", windows)))]
        return Self::new_nop();
    }
}

impl Clipboard {
    pub fn store(&mut self, text: impl Into<String>) {
        self.clipboard
            .set_contents(text.into())
            .unwrap_or_else(|err| {
                panic!("Unable to store text in clipboard: {}", err);
            });
    }

    pub fn _load(&mut self) -> String {
        match self.clipboard.get_contents() {
            Err(err) => {
                panic!("Unable to load text from clipboard: {}", err);
            }
            Ok(text) => text,
        }
    }
}
