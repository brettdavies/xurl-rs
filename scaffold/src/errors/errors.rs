use serde_json;
pub const ERR_TYPE_HTTP: String = "HTTP Error";
pub const ERR_TYPE_IO: String = "IO Error";
pub const ERR_TYPE_INVALID_METHOD: String = "Invalid Method";
pub const ERR_TYPE_API: String = "API Error";
pub const ERR_TYPE_JSON: String = "JSON Error";
pub const ERR_TYPE_AUTH: String = "Auth Error";
pub const ERR_TYPE_TOKEN_STORE: String = "Token Store Error";
#[derive(Debug, Clone, Default)]
pub struct Error {
    pub r#type: String,
    pub message: String,
    pub(crate) cause: anyhow::Error,
}
pub fn new_error(error_type: &str, message: &str, cause: anyhow::Error) -> Box<Error> {
    Box::new(Error {
        r#type: error_type,
        message: message,
        cause: cause,
        ..Default::default()
    })
}
pub fn new_http_error(cause: anyhow::Error) -> Box<Error> {
    new_error(err_type_http, cause.error(), cause)
}
pub fn new_io_error(cause: anyhow::Error) -> Box<Error> {
    new_error(err_type_io, cause.error(), cause)
}
pub fn new_invalid_method_error(method: &str) -> Box<Error> {
    new_error(err_type_invalid_method, format!("Invalid HTTP method: {}", method), None)
}
pub fn new_api_error(data: serde_json::Value) -> Box<Error> {
    new_error(err_type_api, String::from(data), None)
}
pub fn new_json_error(cause: anyhow::Error) -> Box<Error> {
    new_error(err_type_json, cause.error(), cause)
}
pub fn new_auth_error(message: &str, cause: anyhow::Error) -> Box<Error> {
    new_error(err_type_auth, message, cause)
}
pub fn new_token_store_error(message: &str) -> Box<Error> {
    new_error(err_type_token_store, message, None)
}
pub fn is_error_type(err: anyhow::Error, error_type: &str) -> bool {
    let mut e: Box<Error> = Default::default();
    let mut ok = err.downcast_ref::<&e>();
    if ok {
        e.r#type == error_type
    }
    false
}
pub fn is_http_error(err: anyhow::Error) -> bool {
    is_error_type(err, err_type_http)
}
pub fn is_io_error(err: anyhow::Error) -> bool {
    is_error_type(err, err_type_io)
}
pub fn is_api_error(err: anyhow::Error) -> bool {
    is_error_type(err, err_type_api)
}
pub fn is_json_error(err: anyhow::Error) -> bool {
    is_error_type(err, err_type_json)
}
pub fn is_auth_error(err: anyhow::Error) -> bool {
    is_error_type(err, err_type_auth)
}
impl Error {
    pub fn error(&mut self) -> String {
        let mut js: serde_json::Value = Default::default();
        if serde_json::from_str(&self.message.as_bytes().to_vec()).is_none() {
            String::from(js)
        }
        if self.cause.is_some() {
            format!("{}: {} (cause: {})", self.r#type, self.message, self.cause)
        }
        format!("{}: {}", self.r#type, self.message)
    }
    pub fn unwrap(&mut self) -> anyhow::Result<()> {
        Ok(self.cause)
    }
    pub fn is(&mut self, target: anyhow::Error) -> bool {
        let (t, ok) = target.downcast_ref::<Box<Error>>();
        if !ok {
            false
        }
        self.r#type == t.r#type
    }
}
