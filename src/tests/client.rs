use crate::client::Client;

#[test]
#[cfg(not(feature = "plugin"))]
fn init_client_without_plugin(){
    use crate::client;
    let client = client("foo1", "foo2", "foo3", "foo4");

    let buf = [0x10, 0x11, 0x12, 0x13];
    assert!(!client.infer.is_custom(&buf));
}

#[test]
fn set_bucket_name(){
    use crate::client;
    let mut client = client("a","b","c","d");
    client.set_bucket_name("abcaaa".to_owned().into());

    assert_eq!(client.bucket.as_ref(), "abcaaa");
}

mod test_use_plugin{
    #[cfg(feature = "plugin")]
    #[test]
    fn test_install_plugin(){
        use std::sync::Mutex;
        use crate::client;

        //#[mockall_double::double]
        use crate::plugin::{MockPlugin, MockPluginStore};

        let mut plugin_store = MockPluginStore::new();

        plugin_store.expect_insert().times(1).returning(|_|());
        
        let mut client = client("foo1","foo2","foo3","foo4");

        client.plugins = Mutex::new(plugin_store);

        let mut plugin = MockPlugin::new();
        plugin.expect_initialize().times(1)
            .returning(|_|Ok(()));
        
        plugin.expect_name().times(0).returning(||"foo_plugin");
      
        let res = client.plugin(Box::new(plugin));
        assert!(res.is_ok());
    }
}


#[test]
fn test_get_bucket_url(){
    let client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        "foo3".to_owned().into(),
        "foo4".to_owned().into()
    );
    let result = client.get_bucket_url();
    assert!(result.is_err());

    let client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        "https://fobar.example.net".to_owned().into(),
        "foo4".to_owned().into()
    );
    let result = client.get_bucket_url();
    assert!(result.is_ok());

    let url = result.unwrap().to_string();
    assert_eq!(url, "https://foo4.fobar.example.net/".to_string());
}


mod handle_error{
    use futures::executor::block_on;
    use reqwest::Response;
    use http::Response as HttpResponse;
    use crate::client::AsyncRequestHandle;
    use crate::errors::OssError;

    #[test]
    fn test_call_async(){
        block_on(test_async_has_error());
        block_on(test_async_ok());
    }
    
    async fn test_async_has_error(){
        use mockall::*;
        #[mockall_double::double]
        use crate::errors::OssService;

        let mock = OssService::new_context();
        mock.expect()
            .with(predicate::eq("body_abc".to_string()))
            .times(1)
            .returning(move|_x|{
                crate::errors::OssService{
                    code: "foo_code".to_string(),
                    message: "bar".to_string(),
                    request_id: "bar_id".to_string(),
                }
            });

        let http = HttpResponse::builder()
            .status(302)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, OssError::OssService(_)));
        assert!(matches!(err, OssError::OssService(x) if x.code=="foo_code"));

        mock.checkpoint();
    }

    async fn test_async_ok(){
        use mockall::*;
        #[mockall_double::double]
        use crate::errors::OssService;

        let mock = OssService::new_context();
        mock.expect()
            .with(predicate::eq("body_abc".to_string()))
            .times(0)
            .returning(move|_x|{
                crate::errors::OssService{
                    code: "foo_code".to_string(),
                    message: "bar".to_string(),
                    request_id: "bar_id".to_string(),
                }
            });
        
        let http = HttpResponse::builder()
            .status(200)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(ok.text().await.unwrap(), "body_abc".to_string());

        let http = HttpResponse::builder()
            .status(204)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 204);
        assert_eq!(ok.text().await.unwrap(), "body_abc".to_string());

        mock.checkpoint();
    }

    // #[cfg(feature = "blocking")]
    // #[test]
    // fn test_blocking_has_error(){
    //     use reqwest::blocking::Response;
    //     use crate::client::ReqeustHandler;
    //     use mockall::*;
    //     #[mockall_double::double]
    //     use crate::errors::OssService;

    //     let mock = OssService::new_context();
    //     mock.expect()
    //         .with(predicate::eq("body_abc".to_string()))
    //         .times(1)
    //         .returning(move|_x|{
    //             crate::errors::OssService{
    //                 code: "foo_code".to_string(),
    //                 message: "bar".to_string(),
    //                 request_id: "bar_id".to_string(),
    //             }
    //         });

    //     let http = HttpResponse::builder()
    //         .status(302)
    //         //.header("X-Custom-Foo", "Bar")
    //         .body("body_abc")
    //         .unwrap();
    //     let response: Response = http.into();

    //     let res = response.handle_error();

    //     assert!(res.is_err());
    //     let err = res.unwrap_err();
    //     assert!(matches!(err, OssError::OssService(_)));
    //     assert!(matches!(err, OssError::OssService(x) if x.code=="foo_code"));

    //     mock.checkpoint();
    // }

    // #[cfg(feature = "blocking")]
    // #[test]
    // fn test_blocking_ok(){
    //     use reqwest::blocking::Response;
    //     use crate::client::ReqeustHandler;
    //     use mockall::*;
    //     #[mockall_double::double]
    //     use crate::errors::OssService;

    //     let mock = OssService::new_context();
    //     mock.expect()
    //         .with(predicate::eq("body_abc".to_string()))
    //         .times(0)
    //         .returning(move|_x|{
    //             crate::errors::OssService{
    //                 code: "foo_code".to_string(),
    //                 message: "bar".to_string(),
    //                 request_id: "bar_id".to_string(),
    //             }
    //         });
        
    //     let http = HttpResponse::builder()
    //         .status(200)
    //         //.header("X-Custom-Foo", "Bar")
    //         .body("body_abc")
    //         .unwrap();
    //     let response: Response = http.into();

    //     let res = response.handle_error();
    //     assert!(res.is_ok());
    //     let ok = res.unwrap();
    //     assert_eq!(ok.status(), 200);
    //     assert_eq!(ok.text().unwrap(), "body_abc".to_string());

    //     let http = HttpResponse::builder()
    //         .status(204)
    //         //.header("X-Custom-Foo", "Bar")
    //         .body("body_abc")
    //         .unwrap();
    //     let response: Response = http.into();

    //     let res = response.handle_error();
    //     assert!(res.is_ok());
    //     let ok = res.unwrap();
    //     assert_eq!(ok.status(), 204);
    //     assert_eq!(ok.text().unwrap(), "body_abc".to_string());

    //     mock.checkpoint();
    // }

}

// blocking mock 有错误
