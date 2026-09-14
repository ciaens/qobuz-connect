//! The Qobuz Connect token that authenticates a cloud connection, minted by the Qobuz API for a logged-in user.

use serde::Deserialize;

use crate::Error;
use crate::transport::Credentials;

/// Where the official apps mint their tokens.
pub const TOKEN_URL: &str = "https://www.qobuz.com/api.json/0.2/qws/createToken";

/// A token request for any HTTP client: POST `body` as `application/x-www-form-urlencoded` to `url` with `headers`, then read the response with `Credentials::from_json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRequest {
    pub url: &'static str,
    pub headers: [(&'static str, String); 2],
    pub body: &'static str,
}

impl TokenRequest {
    /// Takes the app id and user auth token the Qobuz API wants on every call of a logged-in user.
    #[must_use]
    pub fn new(app_id: &str, user_auth_token: &str) -> Self {
        Self {
            url: TOKEN_URL,
            headers: [
                ("X-App-Id", app_id.to_owned()),
                ("X-User-Auth-Token", user_auth_token.to_owned()),
            ],
            body: "jwt=jwt_qws",
        }
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    jwt_qws: Option<Token>,
}

#[derive(Deserialize)]
struct Token {
    jwt: String,
    endpoint: String,
}

impl Credentials {
    /// Reads the response to a `TokenRequest`.
    pub fn from_json(body: &[u8]) -> Result<Self, Error> {
        let response: TokenResponse =
            serde_json::from_slice(body).map_err(|err| Error::Token(err.to_string()))?;
        let token = response
            .jwt_qws
            .ok_or_else(|| Error::Token("no jwt_qws in the response".to_owned()))?;
        Ok(Self {
            endpoint: token.endpoint,
            jwt: token.jwt,
        })
    }
}
