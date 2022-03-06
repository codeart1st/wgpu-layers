use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, WebHandle};

pub struct OffscreenWindow {
  handle: WebHandle
}

impl OffscreenWindow {
  pub fn new() -> Self {
    Self {
      // currently not needed inside web worker
      handle: WebHandle::empty()
    }
  }
}

unsafe impl HasRawWindowHandle for OffscreenWindow {
  fn raw_window_handle(&self) -> RawWindowHandle {
    RawWindowHandle::Web(self.handle)
  }
}