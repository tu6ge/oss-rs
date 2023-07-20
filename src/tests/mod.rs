#[cfg(feature = "core")]
mod client;

#[cfg(feature = "core")]
mod errors;

#[cfg(feature = "core")]
pub mod object;

#[cfg(feature = "env_test")]
mod env;

pub async fn reqwest_error() -> reqwest::Error {
    use http::response::Builder;
    use reqwest::Response;
    use serde::Deserialize;

    let response = Builder::new().status(200).body("aaaa").unwrap();

    #[derive(Debug, Deserialize)]
    struct Ip;

    let response = Response::from(response);
    response.json::<Ip>().await.unwrap_err()
}
