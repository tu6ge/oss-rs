use std::env::{remove_var, set_var};

use crate::{
    builder::ClientWithMiddleware,
    client::Client,
    config::{BucketBase, Config, InvalidConfigKind},
    types::EndPointKind,
    EndPoint,
};

#[test]
fn client_from_env() {
    set_var("ALIYUN_KEY_ID", "foo1");
    set_var("ALIYUN_KEY_SECRET", "foo2");
    set_var("ALIYUN_ENDPOINT", "qingdao");
    set_var("ALIYUN_BUCKET", "foo4");
    remove_var("ALIYUN_OSS_INTERNAL");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(url.as_str(), "https://foo4.oss-cn-qingdao.aliyuncs.com/");

    set_var("ALIYUN_OSS_INTERNAL", "true");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(
        url.as_str(),
        "https://foo4.oss-cn-qingdao-internal.aliyuncs.com/"
    );

    set_var("ALIYUN_OSS_INTERNAL", "foo4");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(url.as_str(), "https://foo4.oss-cn-qingdao.aliyuncs.com/");

    set_var("ALIYUN_OSS_INTERNAL", "1");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(
        url.as_str(),
        "https://foo4.oss-cn-qingdao-internal.aliyuncs.com/"
    );

    set_var("ALIYUN_OSS_INTERNAL", "yes");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(
        url.as_str(),
        "https://foo4.oss-cn-qingdao-internal.aliyuncs.com/"
    );

    set_var("ALIYUN_OSS_INTERNAL", "Y");
    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    let url = client.get_bucket_url();
    assert_eq!(
        url.as_str(),
        "https://foo4.oss-cn-qingdao-internal.aliyuncs.com/"
    );
}

#[test]
fn config_from_env() {
    set_var("ALIYUN_KEY_ID", "foo");
    set_var("ALIYUN_KEY_SECRET", "foo2");
    set_var("ALIYUN_ENDPOINT", "qingdao");
    set_var("ALIYUN_BUCKET", "foo3");
    remove_var("ALIYUN_OSS_INTERNAL");
    let config = Config::from_env().unwrap();
    assert_eq!(config.key.as_ref(), "foo");
    assert_eq!(config.secret.as_str(), "foo2");
    assert_eq!(&config.endpoint, &EndPoint::CN_QINGDAO);
    assert_eq!(config.bucket.as_ref(), "foo3");

    set_var("ALIYUN_ENDPOINT", "ossqd");
    let config = Config::from_env().unwrap_err();
    assert!(config.clone().get_source().len() == 0);
    assert!(matches!(config.kind(), InvalidConfigKind::EndPoint(_)));

    set_var("ALIYUN_ENDPOINT", "hangzhou");
    set_var("ALIYUN_BUCKET", "foo3-");
    let config = Config::from_env().unwrap_err();
    assert!(config.clone().get_source().len() == 0);
    assert!(matches!(config.kind(), InvalidConfigKind::BucketName(_)));
}

#[test]
fn bucket_base_from_env() {
    set_var("ALIYUN_ENDPOINT", "qingdao");
    set_var("ALIYUN_BUCKET", "foo1");

    set_var("ALIYUN_OSS_INTERNAL", "0");
    let base = BucketBase::from_env().unwrap();
    assert!(!base.endpoint().is_internal());

    set_var("ALIYUN_OSS_INTERNAL", "1");
    let base = BucketBase::from_env().unwrap();
    assert!(base.endpoint().is_internal());

    set_var("ALIYUN_OSS_INTERNAL", "yes");
    let base = BucketBase::from_env().unwrap();
    assert!(base.endpoint().is_internal());

    set_var("ALIYUN_OSS_INTERNAL", "Y");
    let base = BucketBase::from_env().unwrap();
    assert!(base.endpoint().is_internal());

    remove_var("ALIYUN_OSS_INTERNAL");
    remove_var("ALIYUN_ENDPOINT");
    let base = BucketBase::from_env().unwrap_err();
    assert_eq!(base.clone().get_source(), "ALIYUN_ENDPOINT");
    assert!(matches!(base.kind(), InvalidConfigKind::VarError(_)));

    set_var("ALIYUN_ENDPOINT", "ossqd");
    let base = BucketBase::from_env().unwrap_err();
    assert_eq!(base.clone().get_source(), "ossqd");
    assert!(matches!(base.kind(), InvalidConfigKind::EndPoint(_)));

    set_var("ALIYUN_ENDPOINT", "qingdao");
    remove_var("ALIYUN_BUCKET");
    let base = BucketBase::from_env().unwrap_err();
    assert_eq!(base.clone().get_source(), "ALIYUN_BUCKET");
    assert!(matches!(base.kind(), InvalidConfigKind::VarError(_)));

    set_var("ALIYUN_BUCKET", "abc-");
    let base = BucketBase::from_env().unwrap_err();
    assert_eq!(base.clone().get_source(), "abc-");
    assert!(matches!(base.kind(), InvalidConfigKind::BucketName(_)));
}

#[test]
fn end_point_from_env() {
    remove_var("ALIYUN_ENDPOINT");
    let has_err = EndPoint::from_env();
    assert!(has_err.is_err());

    set_var("ALIYUN_ENDPOINT", "ossaa");
    let has_err = EndPoint::from_env();
    assert!(has_err.is_err());

    set_var("ALIYUN_ENDPOINT", "qingdao");
    remove_var("ALIYUN_OSS_INTERNAL");
    let endpoint = EndPoint::from_env().unwrap();
    assert_eq!(endpoint.kind, EndPointKind::CnQingdao);
    assert!(!endpoint.is_internal);

    set_var("ALIYUN_OSS_INTERNAL", "true");
    let endpoint = EndPoint::from_env().unwrap();
    assert_eq!(endpoint.kind, EndPointKind::CnQingdao);
    assert!(endpoint.is_internal);

    set_var("ALIYUN_OSS_INTERNAL", "0");
    let endpoint = EndPoint::from_env().unwrap();
    assert!(!endpoint.is_internal);

    set_var("ALIYUN_OSS_INTERNAL", "1");
    let endpoint = EndPoint::from_env().unwrap();
    assert!(endpoint.is_internal);

    set_var("ALIYUN_OSS_INTERNAL", "yes");
    let endpoint = EndPoint::from_env().unwrap();
    assert!(endpoint.is_internal);

    set_var("ALIYUN_OSS_INTERNAL", "Y");
    let endpoint = EndPoint::from_env().unwrap();
    assert!(endpoint.is_internal);
}
