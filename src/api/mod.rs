/// X API client — request building, response handling, shortcuts, and media.
mod endpoints;
mod media;
mod request;
pub mod response;
mod shortcuts;

pub use endpoints::is_streaming_endpoint;
pub use media::{
    execute_media_status, execute_media_upload, extract_media_id, extract_segment_index,
    handle_media_append_request, is_media_append_request, MEDIA_ENDPOINT,
};
pub use request::{ApiClient, MultipartOptions, RequestOptions};
pub use shortcuts::*;
