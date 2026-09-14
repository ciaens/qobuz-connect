#![allow(clippy::unwrap_used, clippy::panic)]

use qobuz_connect::{Credentials, Error, TokenRequest};

#[test]
fn the_request_matches_the_web_player() {
    let request = TokenRequest::new("app", "user-token");
    assert_eq!(
        request.url,
        "https://www.qobuz.com/api.json/0.2/qws/createToken"
    );
    assert_eq!(
        request.headers,
        [
            ("X-App-Id", "app".to_owned()),
            ("X-User-Auth-Token", "user-token".to_owned()),
        ]
    );
    assert_eq!(request.body, "jwt=jwt_qws");
}

#[test]
fn the_response_yields_credentials() {
    let body = br#"{"jwt_qws":{"jwt":"eyJhbGciOiJIUzI1NiJ9.e30.x","endpoint":"wss:\/\/qws-eu-prod.qobuz.com\/ws"}}"#;
    let credentials = Credentials::from_json(body).unwrap();
    assert_eq!(credentials.endpoint, "wss://qws-eu-prod.qobuz.com/ws");
    assert_eq!(credentials.jwt, "eyJhbGciOiJIUzI1NiJ9.e30.x");
}

#[test]
fn a_response_without_a_token_is_an_error() {
    assert!(matches!(
        Credentials::from_json(br#"{"jwt_qws":null}"#),
        Err(Error::Token(_))
    ));
    assert!(matches!(
        Credentials::from_json(b"not json"),
        Err(Error::Token(_))
    ));
}
