use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, WebHandle};

pub struct OffscreenWindow {
  handle: WebHandle
}

impl OffscreenWindow {
  pub fn new() -> Self {
    let mut handle = WebHandle::empty();
    handle.id = 1;

    Self {
      handle
    }
  }
}

unsafe impl HasRawWindowHandle for OffscreenWindow {
  fn raw_window_handle(&self) -> RawWindowHandle {
    RawWindowHandle::Web(self.handle)
  }
}