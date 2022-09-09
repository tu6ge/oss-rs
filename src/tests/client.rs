use std::collections::HashMap;
use reqwest::Url;
use crate::client::Client;

mod test_async_canonicalized_resource{

    use reqwest::Url;
    use crate::client::Client;

    #[cfg(feature = "plugin")]
    #[tokio::test]
    async fn test_plugin(){
        use crate::plugin::Plugin;
        // 创建一个扩展 struct
        struct SimplePlugin;

        // 实现 Plugin trait 中的方法，
        impl Plugin for SimplePlugin {
            fn name(&self) -> &'static str {
                "bar"
            }

            fn canonicalized_resource(&self, _url: &reqwest::Url) -> Option<String> {
                Some("foo_string".to_string())
            }
        }

        let client = Client::new("foo1", "foo2", "foo3", "foo4")
          .plugin(Box::new(SimplePlugin{})).unwrap();
        let url = Url::parse("https://example.net").unwrap();
        
        let resource = client.async_canonicalized_resource(&url, Some("bucket_foo".to_string())).await;

        assert_eq!(resource, "foo_string".to_string());
    }

    #[tokio::test]
    async fn test_empty_bucket(){
        let client = Client::new("foo1", "foo2", "foo3", "");

        let url = Url::parse("https://example.net").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/");

        let resource = client.async_canonicalized_resource(&url, Some("".to_string())).await;
        assert_eq!(resource, "/");
    }

    #[tokio::test]
    async fn test_has_path(){
        let client = Client::new("foo1", "foo2", "foo3", "foo4");

        let url = Url::parse("https://example.net/bar_path").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/bar_path");

        let url = Url::parse("https://example.net/bar_path").unwrap();
        let resource = client.async_canonicalized_resource(&url, Some("bucket_foo".to_string())).await;
        assert_eq!(resource, "/bucket_foo/bar_path");
    }

    #[tokio::test]
    async fn test_has_path_query(){
        let client = Client::new("foo1", "foo2", "foo3", "foo4");

        let url = Url::parse("https://example.net/bar_path?abc=2").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/bar_path?abc=2");

        let url = Url::parse("https://example.net/bar_path?abc=2").unwrap();
        let resource = client.async_canonicalized_resource(&url, Some("bucket_foo".to_string())).await;
        assert_eq!(resource, "/bucket_foo/bar_path?abc=2");
    }

    #[tokio::test]
    async fn test_not_path(){
        let client = Client::new("foo1", "foo2", "foo3", "foo4");

        let url = Url::parse("https://example.net/?acl").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/?acl");

        let url = Url::parse("https://example.net/?bucketInfo").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/?bucketInfo");

        let url = Url::parse("https://foo4.example.net/?continuation-token=fooxxx").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/?continuation-token=fooxxx");

        let url = Url::parse("https://foo4.example.net/?abc").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/foo4/");

        let url = Url::parse("https://fobar.example.net").unwrap();
        let resource = client.async_canonicalized_resource(&url, None).await;
        assert_eq!(resource, "/");
    }

    
}

#[test]
fn test_get_bucket_url(){
    let client = Client::new("foo1", "foo2", "foo3", "foo4");
    let result = client.get_bucket_url();
    assert!(result.is_err());

    let client = Client::new("foo1", "foo2", "https://fobar.example.net", "foo4");
    let result = client.get_bucket_url();
    assert!(result.is_ok());

    let url = result.unwrap().to_string();
    assert_eq!(url, "https://foo4.fobar.example.net/".to_string());
}

#[test]
fn test_is_bucket_url(){
    let client = Client::new("foo1", "foo2", "foo3", "foo4");
    let url = Url::parse("https://foo_bucket.example.net/abc").unwrap();
    let bucket = "foo_bucket".to_string();
    assert!(client.is_bucket_url(&url, &bucket));

    let url = Url::parse("https://foo2.foo_bucket.net/abc").unwrap();
    let bucket = "foo_bucket".to_string();
    let bucket_real = "foo2".to_string();
    assert!(!client.is_bucket_url(&url, &bucket));
    assert!(client.is_bucket_url(&url, &bucket_real));

    let url = Url::parse("https://foo2.example.net/foo_bucket").unwrap();
    let bucket = "foo_bucket".to_string();
    assert!(!client.is_bucket_url(&url, &bucket));
}

#[test]
fn test_object_list_query_generator(){
    use crate::client::Client;

    let query: HashMap<String, String> = HashMap::new();
    let res = Client::<'_>::object_list_query_generator(&query);

    assert_eq!(res, "list-type=2".to_owned());

    let mut query: HashMap<String, String> = HashMap::new();
    query.insert("key1".to_owned(), "val1".to_owned());
    let res = Client::<'_>::object_list_query_generator(&query);

    assert_eq!(res, "list-type=2&key1=val1".to_owned());
}

mod handle_error{
    use reqwest::Response;
    use http::Response as HttpResponse;
    use crate::client::AsyncRequestHandle;
    use crate::errors::OssError;
    
    #[tokio::test]
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

    #[tokio::test]
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

}

// blocking mock 有错误
