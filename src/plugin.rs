//! # 插件
//! 使用插件改变库的默认行为
//! 增加库的可扩展性
//! 
use std::collections::HashMap;
use std::ops::ControlFlow;

use reqwest::Url;
use serde_json::{Value as JsonValue};

use crate::errors::OssError;

pub type Result<T> = std::result::Result<T, OssError>;

pub trait Plugin{
  fn name(&self) -> &'static str;

  /// Initializes the plugin.
  #[allow(unused_variables)]
  fn initialize(&mut self, config: JsonValue) -> Result<()> {
    Ok(())
  }

  #[allow(unused_variables)]
  fn canonicalized_resource(&self, url: &Url) -> Option<String> {
    None
  }
}

pub struct PluginStore{
  store: HashMap<&'static str, Box<dyn Plugin>>,
}

impl Default for PluginStore {
  fn default() -> Self {
    Self {
      store: HashMap::new(),
    }
  }
}

impl PluginStore {
  pub fn insert(&mut self, plugin: Box<dyn Plugin>){
    let name = plugin.name();
    self.store.insert(name, plugin);
  }

  pub fn get_canonicalized_resource(&self, url: &Url) -> Option<String> {
    let result = self.store.values().try_for_each(move|plugin| {
      let canonicalized_resource = plugin.canonicalized_resource(url);
      if let Some(val) = canonicalized_resource {
        return ControlFlow::Break(val)
      }
      ControlFlow::Continue(())
    });

    match result {
      ControlFlow::Break(val) => Some(val),
      _ => None,
    }
  }
}