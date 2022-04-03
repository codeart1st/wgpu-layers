use geo_types::GeometryCollection;
use log::warn;
use prost::Message;

use crate::bucket::feature::Feature;

mod vector_tile {
  include!(concat!(env!("OUT_DIR"), "/vector_tile.rs"));
}

pub struct Parser {
  tile: vector_tile::Tile,
}

impl Parser {
  pub fn new(data: Vec<u8>) -> Result<Self, prost::DecodeError> {
    Ok(Self {
      tile: vector_tile::Tile::decode(prost::bytes::Bytes::from(data))?,
    })
  }

  pub fn get_layer_names(&self) -> Vec<String> {
    let mut layer_names = Vec::with_capacity(self.tile.layers.len());
    for layer in self.tile.layers.iter() {
      match layer.version {
        1 | 2 => {
          layer_names.push(layer.name.clone());
        }
        _ => {
          warn!(
            "Vector tile version not supported for layer `{}` (found version: {})",
            layer.name, layer.version
          );
        }
      }
    }
    layer_names
  }

  pub fn get_features(&self, layer_index: usize) -> Option<Vec<Feature<GeometryCollection<f32>>>> {
    let layer = self.tile.layers.get(layer_index);
    match layer {
      Some(layer) => {
        let mut features = Vec::with_capacity(layer.features.len());
        for feature in layer.features.iter() {
          features.push(Feature {
            geometry: GeometryCollection::<f32>::new(),
            properties: Some(parse_tags(&feature.tags, &layer.keys, &layer.values)),
          });
        }
        Some(features)
      }
      None => None,
    }
  }
}

fn parse_tags(
  tags: &[u32],
  keys: &[String],
  values: &[vector_tile::tile::Value],
) -> std::collections::HashMap<String, String> {
  tags
    .chunks(2)
    .fold(std::collections::HashMap::new(), |mut acc, item| {
      acc.insert(
        (*keys.get(item[0] as usize).expect("item not found")).clone(),
        get_string_value((*values.get(item[1] as usize).expect("item not found")).clone()),
      );
      acc
    })
}

fn get_string_value(value: vector_tile::tile::Value) -> String {
  if value.string_value.is_some() {
    return value.string_value.unwrap();
  }
  if value.float_value.is_some() {
    return value.float_value.unwrap().to_string();
  }
  if value.double_value.is_some() {
    return value.double_value.unwrap().to_string();
  }
  if value.int_value.is_some() {
    return value.int_value.unwrap().to_string();
  }
  if value.uint_value.is_some() {
    return value.uint_value.unwrap().to_string();
  }
  if value.sint_value.is_some() {
    return value.sint_value.unwrap().to_string();
  }
  if value.bool_value.is_some() {
    return value.bool_value.unwrap().to_string();
  }
  String::new()
}
