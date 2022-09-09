
#[test]
fn test_insert(){
    #[mockall_double::double]
    use crate::plugin::Plugin;
    use crate::plugin::PluginStore;

    let mut plugin_store = PluginStore::default();
    let mut plugin = Plugin::default();
    plugin.expect_name().times(2).returning(||"foo_plugin");

    plugin_store.insert(Box::new(plugin));

    let store = plugin_store.store();

    assert_eq!(store.len(), 1);
    assert!(store.contains_key("foo_plugin"));
    assert!(store.get("foo_plugin").is_some());
    assert_eq!(store.get("foo_plugin").unwrap().name(), "foo_plugin");
}

#[test]
fn test_initialize(){
    #[mockall_double::double]
    use crate::plugin::Plugin;
    use crate::plugin::PluginStore;
    use crate::client::Client;

    let mut plugin_store = PluginStore::default();
    let mut plugin = Plugin::default();
    plugin.expect_name().times(1).returning(||"foo_plugin");
    plugin.expect_initialize().times(1).returning(|_|Ok(()));

    plugin_store.insert(Box::new(plugin));

    let mut client = Client::new("foo1", "foo2", "foo3", "foo4");

    let res = plugin_store.initialize(&mut client);

    assert!(res.is_ok());
}

#[test]
fn test_initialize_with_plugin_error(){
    #[mockall_double::double]
    use crate::plugin::Plugin;
    use crate::plugin::PluginStore;
    use crate::client::Client;
    use crate::errors::OssError;

    let mut plugin_store = PluginStore::default();
    let mut plugin = Plugin::default();
    plugin.expect_name().times(2).returning(||"foo_plugin");
    plugin.expect_initialize().times(1).returning(|_|Err(OssError::Input("foo_error".to_string())));

    plugin_store.insert(Box::new(plugin));

    let mut client = Client::new("foo1", "foo2", "foo3", "foo4");

    let res = plugin_store.initialize(&mut client);

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(err, OssError::Plugin(_)));
    assert!(matches!(err, OssError::Plugin(p) if p.name=="foo_plugin"));
}

mod test_canonicalized{
    #[mockall_double::double]
    use crate::plugin::Plugin;
    use crate::plugin::PluginStore;
    use reqwest::Url;

    #[test]
    fn test_none(){
        let url = Url::parse("https://example.net/").unwrap();
    
        let mut plugin_store = PluginStore::default();
        let mut plugin = Plugin::default();
        plugin.expect_name().times(1).returning(||"foo_plugin");
        plugin.expect_canonicalized_resource()
            .times(1)
            .withf(|u|u.as_str()=="https://example.net/")
            .returning(|_|None);
    
        plugin_store.insert(Box::new(plugin));
    
        let res = plugin_store.get_canonicalized_resource(&url);
        assert!(res.is_none());
    }

    #[test]
    fn test_some_value(){
        let url = Url::parse("https://example.net/").unwrap();
    
        let mut plugin_store = PluginStore::default();
        let mut plugin = Plugin::default();
        plugin.expect_name().times(1).returning(||"foo_plugin");
        plugin.expect_canonicalized_resource()
            .times(1)
            .withf(|u|u.as_str()=="https://example.net/")
            .returning(|_|Some("foo_val".to_string()));
    
        plugin_store.insert(Box::new(plugin));
    
        let res = plugin_store.get_canonicalized_resource(&url);
        assert!(res.is_some());
        let val = res.unwrap();
        assert_eq!(val, "foo_val".to_string());
    }

    // TODO 未稳定
    // #[test]
    // fn test_some_and_none(){
    //     let url = Url::parse("https://example.net/").unwrap();
    
    //     let mut plugin_store = PluginStore::default();
    //     let mut plugin = Plugin::default();
    //     plugin.expect_name().times(1).returning(||"foo_plugin");
    //     plugin.expect_canonicalized_resource()
    //         .times(1)
    //         .withf(|u|u.as_str()=="https://example.net/")
    //         .returning(|_|Some("foo_val_some_and_none".to_string()));

    //     let mut plugin2 = Plugin::default();
    //     plugin2.expect_name().times(1).returning(||"foo_plugin2");
    //     plugin2.expect_canonicalized_resource()
    //         .never()
    //         .withf(|u|u.as_str()=="https://example.net/")
    //         .returning(|_|None);
    
    //     plugin_store.insert(Box::new(plugin));
    //     plugin_store.insert(Box::new(plugin2));
    
    //     let res = plugin_store.get_canonicalized_resource(&url);
    //     assert!(res.is_some());
    //     let val = res.unwrap();
    //     assert_eq!(val, "foo_val_some_and_none".to_string());
    // }

    // #[test]
    // fn test_none_and_some(){
    //     let url = Url::parse("https://example.net/").unwrap();
    
    //     let mut plugin_store = PluginStore::default();
    //     let mut plugin = Plugin::default();
    //     plugin.expect_name().times(1).returning(||"foo_plugin");
    //     plugin.expect_canonicalized_resource()
    //         .times(1)
    //         .withf(|u|u.as_str()=="https://example.net/")
    //         .returning(|_|None);

    //     let mut plugin2 = Plugin::default();
    //     plugin2.expect_name().times(1).returning(||"foo_plugin2");
    //     plugin2.expect_canonicalized_resource()
    //         .times(1)
    //         .withf(|u|u.as_str()=="https://example.net/")
    //         .returning(|_|Some("foo_val2".to_string()));
    
    //     plugin_store.insert(Box::new(plugin));
    //     plugin_store.insert(Box::new(plugin2));
    
    //     let res = plugin_store.get_canonicalized_resource(&url);
    //     assert!(res.is_some());
    //     let val = res.unwrap();
    //     assert_eq!(val, "foo_val2".to_string());
    // }
}
