/// `OAuth2` callback server — listens for the authorization code redirect.
///
/// Starts a minimal HTTP server on `127.0.0.1:<port>` that handles the
/// `OAuth2` callback, validates the state parameter, and returns the code.
/// Times out after 5 minutes, matching the Go implementation.
use std::time::Duration;

use tokio::sync::oneshot;

use crate::error::{Result, XurlError};

/// Starts a callback server and waits for the authorization code.
///
/// Returns the authorization code on success, or an error on timeout/failure.
pub fn wait_for_callback(port: u16, expected_state: &str) -> Result<String> {
    let expected_state = expected_state.to_string();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| XurlError::auth_with_cause("ServerError", &e))?;

    rt.block_on(async {
        let (tx, rx) = oneshot::channel::<std::result::Result<String, String>>();
        let tx = std::sync::Mutex::new(Some(tx));

        let state_clone = expected_state.clone();

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
            .await
            .map_err(|e| XurlError::auth_with_cause("ServerError", &e))?;

        let server_handle = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    continue;
                };

                let tx_ref = &tx;
                let state = state_clone.clone();

                let mut buf = [0u8; 4096];
                let _ = stream.readable().await;
                let Ok(n) = stream.try_read(&mut buf) else {
                    continue;
                };

                let request = String::from_utf8_lossy(&buf[..n]);

                // Parse the GET request for code and state
                let first_line = request.lines().next().unwrap_or("");
                let path = first_line.split_whitespace().nth(1).unwrap_or("");

                if !path.starts_with("/callback") {
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
                    let _ = stream.writable().await;
                    let _ = stream.try_write(response.as_bytes());
                    continue;
                }

                // Parse query parameters
                let query = path.split('?').nth(1).unwrap_or("");
                let params: std::collections::HashMap<&str, &str> = query
                    .split('&')
                    .filter_map(|p| p.split_once('='))
                    .collect();

                let code = params.get("code").unwrap_or(&"");
                let received_state = params.get("state").unwrap_or(&"");

                if *received_state != state {
                    let body = "Error: invalid state parameter";
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.writable().await;
                    let _ = stream.try_write(response.as_bytes());
                    if let Some(tx) = tx_ref.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() {
                        let _ = tx.send(Err("invalid state parameter".to_string()));
                    }
                    break;
                }

                if code.is_empty() {
                    let body = "Error: empty authorization code";
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = stream.writable().await;
                    let _ = stream.try_write(response.as_bytes());
                    if let Some(tx) = tx_ref.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() {
                        let _ = tx.send(Err("empty authorization code".to_string()));
                    }
                    break;
                }

                let body = "Authentication successful! You can close this window.";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.writable().await;
                let _ = stream.try_write(response.as_bytes());

                if let Some(tx) = tx_ref.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() {
                    let _ = tx.send(Ok(code.to_string()));
                }
                break;
            }
        });

        // Wait with 5-minute timeout
        let result = tokio::select! {
            result = rx => {
                match result {
                    Ok(Ok(code)) => Ok(code),
                    Ok(Err(e)) => Err(XurlError::auth(format!("CallbackError: {e}"))),
                    Err(_) => Err(XurlError::auth("ListenerError: oauth2 listener failed")),
                }
            }
            () = tokio::time::sleep(Duration::from_secs(300)) => {
                Err(XurlError::auth("Timeout: authentication timed out"))
            }
        };

        server_handle.abort();
        result
    })
}
