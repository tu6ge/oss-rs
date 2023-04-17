use super::ArcPointer;

#[test]
fn debug_arc_pointer() {
    let pointer = ArcPointer;

    assert_eq!(format!("{pointer:?}"), "ArcPointer");
}

#[test]
#[cfg(feature = "blocking")]
fn debug_rc_pointer() {
    use super::RcPointer;
    let pointer = RcPointer;

    assert_eq!(format!("{pointer:?}"), "RcPointer");
}

mod error {
    use crate::{
        builder::BuilderError, config::InvalidConfig, errors::OssService, types::InvalidEndPoint,
    };

    #[test]
    fn from_oss() {
        let err = BuilderError::OssService(OssService::default());
        assert_eq!(format!("{err}"), "OssService OssService { code: \"Undefined\", status: 200, message: \"Parse aliyun response xml error message failed.\", request_id: \"XXXXXXXXXXXXXXXXXXXXXXXX\" }");

        fn bar() -> BuilderError {
            OssService::default().into()
        }

        assert_eq!(format!("{:?}", bar()), "OssService(OssService { code: \"Undefined\", status: 200, message: \"Parse aliyun response xml error message failed.\", request_id: \"XXXXXXXXXXXXXXXXXXXXXXXX\" })")
    }

    #[test]
    #[cfg(feature = "auth")]
    fn from_auth() {
        use crate::auth::{AuthError, AuthErrorKind};

        let err = BuilderError::AuthError(AuthError {
            kind: AuthErrorKind::InvalidCanonicalizedResource,
        });
        assert_eq!(format!("{err}"), "invalid canonicalized-resource");

        fn bar() -> BuilderError {
            AuthError {
                kind: AuthErrorKind::InvalidCanonicalizedResource,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "AuthError(AuthError { kind: InvalidCanonicalizedResource })"
        );
    }

    #[test]
    fn from_config() {
        let err = BuilderError::Config(InvalidConfig::EndPoint(InvalidEndPoint { _priv: () }));
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        fn bar() -> BuilderError {
            InvalidConfig::EndPoint(InvalidEndPoint { _priv: () }).into()
        }
        assert_eq!(format!("{:?}", bar()), "Config(EndPoint(InvalidEndPoint))");
    }
}
