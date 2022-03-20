pub struct View {
  /// width of surface
  pub width: u32,

  /// height of surface
  pub height: u32,

  /// transformation matrix world-space to view-space
  pub view_matrix: [f32; 16],
}

impl View {
  pub fn new((width, height): (u32, u32)) -> Self {
    Self {
      width,
      height,
      #[rustfmt::skip]
      view_matrix: [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
      ],
    }
  }
}
