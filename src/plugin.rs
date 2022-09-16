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
let client = aliyun_oss_client::client(&key_id,&key_secret, &endpoint, &bucket)
    .plugin(Box::new(my_plugin));
```
 * 
 */

use std::collections::HashMap;
use reqwest::Url;
use crate::{errors::{OssResult, OssError, plugin::PluginError}, client::Client};

#[cfg_attr(test, mockall::automock)]
pub trait Plugin: Send{
  fn name(&self) -> &'static str;

  /// 初始化插件
  #[allow(unused_variables)]
  fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
    Ok(())
  }

  /// # 限制指定的 Url 才可以使用本插件计算 `canonicalized_resource` 值
  /// 
  /// 只有当本方法返回 true 时，才会使用本插件计算 `canonicalized_resource` 值，否则，使用 lib 本身的计算规则
  /// 
  /// **如果同时安装的两个插件都返回 true，系统会提示错误**
  #[allow(unused_variables)]
  fn astrict_resource_url(&self, url: &Url) -> bool {
    false
  }

  /// 修改 lib 内部计算 canonicalized_resource 参数的方式
  /// 鉴于官方对该参数的定义比较模糊，为了增加 lib 库的通用性，所以使用插件对这个参数进行修改
  /// 如果有多个插件对这个参数进行修改，返回第一个已装配的插件结果
  /// 本 trait 对此方法做了默认实现
  /// 
  /// *只有 `astrict_resource_url` 方法返回 true 时，才起作用*
  #[allow(unused_variables)]
  fn canonicalized_resource(&self, url: &Url) -> Option<String> {
    None
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

  /// 计算插件中的 canonicalized_resource 值，并返回
  pub fn get_canonicalized_resource<'a>(&self, url: &'a Url) -> OssResult<Option<String>> {
    let list: Vec<&Box<dyn Plugin>> = self.store.iter().filter(|(_,p)|{
      p.astrict_resource_url(url)
    }).map(|(_,v)|{
      v
    }).collect();

    if list.len() >1 {
      let names: Vec<_> = list.iter().map(|v|v.name().to_string()).collect();
      let name = names.join(",").clone();
      return Err(OssError::Plugin(PluginError { name: "plugin conflict", message: format!("{} 等多个插件匹配到了同一个 Url: {}", name, url)}));
    }

    Ok(match list.get(0) {
      Some(p) => p.canonicalized_resource(url),
      None => None,
    })
  }
}
