
#[test]
fn plugin_default_return(){
    use crate::plugin::Plugin;
    use crate::client::Client;

    struct MyPlugin;

    impl Plugin for MyPlugin{
        fn name(&self) -> &'static str {
            "my_plugin"
        }
    }

    let mut plugin = MyPlugin{};
    let mut client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        "foo3".to_owned().into(),
        "foo4".to_owned().into()
    );
    let res = plugin.initialize(&mut client);
    assert!(res.is_ok());

}

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

    let mut client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        "foo3".to_owned().into(),
        "foo4".to_owned().into()
    );

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

    let mut client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        "foo3".to_owned().into(),
        "foo4".to_owned().into()
    );

    let res = plugin_store.initialize(&mut client);

    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(matches!(err, OssError::Plugin(_)));
    assert!(matches!(err, OssError::Plugin(p) if p.name=="foo_plugin"));
}
