use std::collections::HashMap;

pub trait WithGeometry<T> {
  fn get_geometry(&self) -> &T;
}

#[derive(Default, Debug)]
pub struct Feature<T> {
  pub geometry: T,

  pub properties: Option<HashMap<String, String>>,
}

impl<T> WithGeometry<T> for Feature<T> {
  fn get_geometry(&self) -> &T {
    &self.geometry
  }
}
