pub struct View {
  /// width of surface
  width: u32,

  /// height of surface
  height: u32,

  /// half width of surface
  half_width: f32,

  /// half height of surface
  half_height: f32,

  /// transformation matrix world-space to view-space
  pub view_matrix: [[f32; 4]; 4],
}

impl View {
  pub fn new((width, height): (u32, u32)) -> Self {
    Self {
      width,
      height,
      half_width: width as f32 * 0.5,
      half_height: height as f32 * 0.5,
      #[rustfmt::skip]
      view_matrix: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
      ],
    }
  }

  pub fn set_size(&mut self, (width, height): (u32, u32)) {
    self.width = width;
    self.height = height;
    self.half_width = width as f32 * 0.5;
    self.half_height = height as f32 * 0.5;
  }

  pub fn get_size(&self) -> (u32, u32) {
    (self.width, self.height)
  }

  pub fn get_half_size(&self) -> (f32, f32) {
    (self.half_width, self.half_height)
  }
}
