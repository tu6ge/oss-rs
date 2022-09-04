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
use std::ops::ControlFlow;
use reqwest::Url;
use crate::{errors::{OssResult, OssError, plugin::PluginError}, client::Client};

pub trait Plugin: Send{
  fn name(&self) -> &'static str;

  /// 初始化插件
  #[allow(unused_variables)]
  fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
    Ok(())
  }

  /// 修改 lib 内部计算 canonicalized_resource 参数的方式
  /// 鉴于官方对该参数的定义比较模糊，为了增加 lib 库的通用性，所以使用插件对这个参数进行修改
  /// 如果有多个插件对这个参数进行修改，返回第一个已装配的插件结果
  /// 本 trait 对此方法做了默认实现
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

impl PluginStore {

  /// 安装插件
  pub fn insert(&mut self, plugin: Box<dyn Plugin>){
    let name = plugin.name();
    self.store.insert(name, plugin);
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

#[cfg(test)]
mod tests {
    use crate::{client::{Client}, errors::OssResult};

    use super::Plugin;

    struct SigFile;

    impl Plugin for SigFile {
        fn name(&self) -> &'static str {
          "sig_file_ext"
        }
    
        fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
            let mime_type = "application/pgp-signature";
            let extension = "sig";
            fn m(buf: &[u8]) -> bool {
                return buf.len() >= 3 && buf[0] == 0x64 && buf[1] == 0x57 && buf[2] == 0x35;
            }
            client.infer.add(mime_type, extension, m);
        
            Ok(())
        }
    }

    #[test]
    fn test_init_infer(){
        let client = crate::client("abc", "abc", "abc", "abc");
        let res = client.infer.get("dW50cnVzdGV".as_bytes());

        assert_matches!(res, None);

        let client_ext = crate::client("abc", "abc", "abc", "abc")
          .plugin(Box::new(SigFile{})).unwrap();
        let res = client_ext.infer.get("dW50cnVzdGV".as_bytes()).unwrap();

        assert_matches!(res.mime_type(), "application/pgp-signature");
        assert_matches!(res.extension(), "sig");
    }
}