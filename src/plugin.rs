/*!
# 插件

使用插件改变库的默认行为
增加库的可扩展性

## example 

```ignore
struct MyPlugin {
  bucket: String,
}

impl Plugin for MyPlugin{
  fn name(&self) -> &'static str {
    "my_plugin"
  }
  
  fn initialize(&mut self, client: &mut Client) {
    // 插件可以读取 client 结构体中的值
    self.bucket = String::from(client.endpoint);

    // 插件可以修改 client 结构体中的值
    client.endpoint = "https://oss-cn-shanghai.aliyuncs.com";
  }

  fn canonicalized_resource(&self, _url: &Url) -> Option<String>{
    None
  }
}


let my_plugin = MyPlugin{bucket:"abc".to_string()};
// 装配插件
let client = aliyun_oss_client::client(key_id,key_secret, endpoint, bucket)
    .plugin(Box::new(my_plugin));
```
 * 
 */

use std::collections::HashMap;
use crate::{errors::{OssResult, OssError, plugin::PluginError}, client::Client};

#[cfg_attr(test, mockall::automock)]
pub trait Plugin: Send{
  fn name(&self) -> &'static str;

  /// 初始化插件
  #[allow(unused_variables)]
  fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
    Ok(())
  }
}

/// 插件仓库
#[non_exhaustive]
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

#[cfg_attr(test, mockall::automock)]
impl PluginStore {

  /// 安装插件
  pub fn insert(&mut self, plugin: Box<dyn Plugin>){
    let name = plugin.name();
    self.store.insert(name, plugin);
  }

  pub fn store(self) -> HashMap<&'static str, Box<dyn Plugin>>{
    self.store
  }

  /// Initializes all plugins in the store.
  pub fn initialize(
    &mut self,
    client: &mut Client
  ) -> OssResult<()> {
    self.store.values_mut().try_for_each(|plugin| {
      plugin
        .initialize(
          client
        )
        .map_err(|e| OssError::Plugin(PluginError{name:plugin.name(), message:e.to_string()}))
    })
  }

}
