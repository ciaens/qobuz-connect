use std::io::{IsTerminal as _, Read as _};

use qobuz_connect::{Credentials, TokenRequest};

fn main() {
    let mut stdin = std::io::stdin();
    if !stdin.is_terminal() {
        let mut body = Vec::new();
        if let Err(err) = stdin.read_to_end(&mut body) {
            eprintln!("{err}");
            return;
        }
        match Credentials::from_json(&body) {
            Ok(credentials) => println!(
                "export QOBUZ_CONNECT_ENDPOINT={}\nexport QOBUZ_CONNECT_JWT={}",
                credentials.endpoint, credentials.jwt
            ),
            Err(err) => eprintln!("{err}"),
        }
        return;
    }
    let (Ok(app_id), Ok(user_auth_token)) = (
        std::env::var("QOBUZ_APP_ID"),
        std::env::var("QOBUZ_USER_AUTH_TOKEN"),
    ) else {
        eprintln!(
            "set QOBUZ_APP_ID and QOBUZ_USER_AUTH_TOKEN to print the request, or pipe its response in to read the token"
        );
        return;
    };
    let request = TokenRequest::new(&app_id, &user_auth_token);
    let headers: Vec<String> = request
        .headers
        .iter()
        .map(|(name, value)| format!("-H '{name}: {value}'"))
        .collect();
    println!(
        "curl -sS {} -d '{}' {}",
        request.url,
        request.body,
        headers.join(" ")
    );
}
