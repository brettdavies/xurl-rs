use serde_json;
pub const MEDIA_ENDPOINT: String = "/2/media/upload";
/// MediaUploader handles media upload operations
#[derive(Debug, Clone, Default)]
pub struct MediaUploader {
    pub(crate) client: Client,
    pub(crate) media_id: String,
    pub(crate) file_path: String,
    pub(crate) file_size: i64,
    pub(crate) verbose: bool,
    pub(crate) auth_type: String,
    pub(crate) username: String,
    pub(crate) headers: Vec<String>,
    pub(crate) trace: bool,
}
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InitRequest {
    #[serde(rename = "total_bytes")]
    pub total_bytes: i64,
    #[serde(rename = "media_type")]
    pub media_type: String,
    #[serde(rename = "media_category")]
    pub media_category: String,
}
/// NewMediaUploader creates a new MediaUploader
pub fn new_media_uploader(
    client: Client,
    file_path: &str,
    verbose: bool,
    trace: bool,
    auth_type: &str,
    username: &str,
    headers: &[String],
) -> anyhow::Result<Box<MediaUploader>> {
    let mut file_info = os.stat(file_path)?;
    if !file_info.mode().is_regular() {
        return Err((anyhow::anyhow!("{} is not a regular file", file_path)).into());
    }
    Ok(
        Box::new(MediaUploader {
            client: client,
            file_path: file_path,
            file_size: file_info.size(),
            verbose: verbose,
            auth_type: auth_type,
            username: username,
            headers: headers,
            trace: trace,
            ..Default::default()
        }),
    )
}
pub fn new_media_uploader_without_file(
    client: Client,
    verbose: bool,
    trace: bool,
    auth_type: &str,
    username: &str,
    headers: &[String],
) -> Box<MediaUploader> {
    Box::new(MediaUploader {
        client: client,
        verbose: verbose,
        auth_type: auth_type,
        username: username,
        headers: headers,
        trace: trace,
        ..Default::default()
    })
}
/// ExecuteMediaUpload handles the media upload command execution
pub fn execute_media_upload(
    file_path: &str,
    media_type: &str,
    media_category: &str,
    auth_type: &str,
    username: &str,
    verbose: bool,
    wait_for_processing: bool,
    trace: bool,
    headers: &[String],
    client: Client,
) -> anyhow::Result<()> {
    let mut uploader = new_media_uploader(
        client,
        file_path,
        verbose,
        trace,
        auth_type,
        username,
        headers,
    )?;
    let mut err = uploader.init(media_type, media_category);
    let mut err = uploader.append();
    let mut finalize_response = uploader.finalize()?;
    utils.format_and_print_response(finalize_response);
    if wait_for_processing && media_category.contains("video") {
        let mut processing_response = uploader.wait_for_processing()?;
        utils.format_and_print_response(processing_response);
    }
    print!(
        "\033[32mMedia uploaded successfully! Media ID: {}\033[0m\n", uploader
        .get_media_id()
    );
    Ok(())
}
/// ExecuteMediaStatus handles the media status command execution
pub fn execute_media_status(
    media_id: &str,
    auth_type: &str,
    username: &str,
    verbose: bool,
    wait: bool,
    trace: bool,
    headers: &[String],
    client: Client,
) -> anyhow::Result<()> {
    let mut uploader = new_media_uploader_without_file(
        client,
        verbose,
        trace,
        auth_type,
        username,
        headers,
    );
    uploader.set_media_id(media_id);
    if wait {
        let mut processing_response = uploader.wait_for_processing()?;
        let mut pretty_json = serde_json::to_string_pretty(&processing_response)?;
        println!("{:?}", String::from(pretty_json));
    } else {
        let mut status_response = uploader.check_status()?;
        let mut pretty_json = serde_json::to_string_pretty(&status_response)?;
        println!("{:?}", String::from(pretty_json));
    }
    Ok(())
}
/// HandleMediaAppendRequest handles a media append request with a file
pub fn handle_media_append_request(
    options: RequestOptions,
    media_file: &str,
    client: Client,
) -> anyhow::Result<serde_json::Value> {
    let mut media_id = extract_media_id(options.endpoint);
    if media_id == "" {
        return Err((anyhow::anyhow!("media_id is required for append endpoint")).into());
    }
    let mut segment_index = extract_segment_index(options.data);
    if segment_index == "" {
        segment_index = "0";
    }
    let mut form_fields = std::collections::HashMap::from([
        ("segment_index", segment_index),
    ]);
    let mut multipart_options = MultipartOptions {
        request_options: options,
        form_fields: form_fields,
        file_field: "media".to_string(),
        file_path: media_file,
        file_name: std::path::Path::new(&media_file)
            .file_name()
            .map(|n| n.to_string_lossy().to_string()),
        file_data: vec![],
        ..Default::default()
    };
    let (response, client_err) = client.send_multipart_request(multipart_options);
    if client_err.is_some() {
        return Err((anyhow::anyhow!("append request failed: {}", client_err)).into());
    }
    Ok(response)
}
/// ExtractMediaID extracts media_id from URL or data
pub fn extract_media_id(url: &str) -> String {
    if url == "" {
        ""
    }
    if !url.contains("/2/media/upload") {
        ""
    }
    if url.ends_with("/2/media/upload/initialize") {
        ""
    }
    if url.contains("/2/media/upload/") {
        let mut parts = url.split("/2/media/upload/").collect::<Vec<&str>>();
        if parts.len() > 1 {
            let mut path = parts[1];
            for suffix in vec!["/append", "/finalize"].iter() {
                let mut idx = path.find(suffix);
                if idx != -1 {
                    path[..idx]
                }
            }
        }
    }
    if url.contains("?") {
        let mut query_params = url.split("?").collect::<Vec<&str>>();
        if query_params.len() > 1 {
            let mut params = query_params[1].split("&").collect::<Vec<&str>>();
            for param in params.iter() {
                if param.starts_with("media_id=") {
                    param.split("=").collect::<Vec<&str>>()[1]
                }
            }
        }
    }
    ""
}
/// extracts command from URL
pub fn extract_command(url: &str) -> String {
    if url.contains("/2/media/upload/") {
        let mut parts = url.split("/2/media/upload/").collect::<Vec<&str>>();
        if parts.len() > 1 {
            let mut path = parts[1];
            if path.contains("/append") {
                "append"
            }
            if path.contains("/finalize") {
                "finalize"
            }
            if path == "initialize" {
                "initialize"
            }
        }
        "status"
    }
    ""
}
/// ExtractSegmentIndex extracts segment_index from URL or data
pub fn extract_segment_index(data: &str) -> String {
    let mut json_data: std::collections::HashMap<String, String> = Default::default();
    let mut err = serde_json::from_str(&data.as_bytes().to_vec());
    let (segment_index, ok) = json_data["segment_index"];
    if ok {
        segment_index
    }
    ""
}
/// IsMediaAppendRequest checks if the request is a media append request
pub fn is_media_append_request(url: &str, media_file: &str) -> bool {
    url.contains("/2/media/upload") && url.contains("append") && media_file != ""
}
impl MediaUploader {
    /// Init initializes the media upload
    pub fn init(
        &mut self,
        media_type: &str,
        media_category: &str,
    ) -> anyhow::Result<()> {
        if self.verbose {
            print!("\033[32mInitializing media upload...\033[0m\n");
        }
        let mut final_url = MediaEndpoint + "/initialize";
        let mut body = InitRequest {
            total_bytes: self.file_size,
            media_type: media_type,
            media_category: media_category,
            ..Default::default()
        };
        let mut json_data = serde_json::to_string(&body)?;
        let mut request_options = RequestOptions {
            method: "POST".to_string(),
            endpoint: final_url,
            headers: self.headers,
            data: String::from(json_data),
            auth_type: self.auth_type,
            username: self.username,
            verbose: self.verbose,
            trace: self.trace,
            ..Default::default()
        };
        let (response, client_err) = self.client.send_request(request_options);
        if client_err.is_some() {
            Ok(anyhow::anyhow!("init request failed: {}", client_err))
        }
        let mut init_response: struct_Data_struct_ID_string__json___id_____ExpiresAfterSecs_int__json___expires_after_secs_____MediaKey_string__json___media_key______json___data____ = Default::default();
        let mut err = serde_json::from_str(&response);
        self.media_id = init_response.data.id;
        if self.verbose {
            utils.format_and_print_response(init_response);
        }
        Ok(())
    }
    /// Append uploads the media in chunks
    pub fn append(&mut self) -> anyhow::Result<()> {
        if self.media_id == "" {
            Ok(anyhow::anyhow!("media ID not set, call Init first"))
        }
        if self.verbose {
            print!("\033[32mUploading media in chunks...\033[0m\n");
        }
        let mut file = os.open(self.file_path)?;
        let _defer = scopeguard::guard(
            (),
            |_| {
                file.close();
            },
        );
        let mut chunk_size = 4 * 1024 * 1024;
        let mut buffer = vec![Default::default(); chunk_size];
        let mut segment_index = 0;
        let mut bytes_uploaded = 0 as i64;
        loop {
            let mut bytes_read = file.read(buffer)?;
            if err == io.eof {
                break;
            }
            let mut final_url = media_endpoint + format!("/{}/append", self.media_id);
            let mut form_fields = std::collections::HashMap::from([
                ("segment_index", segment_index.to_string()),
            ]);
            let mut request_options = RequestOptions {
                method: "POST".to_string(),
                endpoint: final_url,
                headers: self.headers,
                data: "".to_string(),
                auth_type: self.auth_type,
                username: self.username,
                verbose: self.verbose,
                trace: self.trace,
                ..Default::default()
            };
            let mut multipart_options = MultipartOptions {
                request_options: request_options,
                form_fields: form_fields,
                file_field: "media".to_string(),
                file_name: std::path::Path::new(&self.file_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string()),
                file_data: buffer[..bytes_read],
                ..Default::default()
            };
            let (_, client_err) = self.client.send_multipart_request(multipart_options);
            if client_err.is_some() {
                Ok(anyhow::anyhow!("append request failed: {}", client_err))
            }
            bytes_uploaded += bytes_read as i64;
            segment_index += 1;
            if self.verbose {
                print!(
                    "\033[33mUploaded {} of {} bytes (%.2f%%)\033[0m\n", bytes_uploaded,
                    self.file_size, bytes_uploaded as f64 / self.file_size as f64 * 100.0
                );
            }
        }
        if self.verbose {
            print!("\033[32mUpload complete!\033[0m\n");
        }
        Ok(())
    }
    /// Finalize finalizes the media upload
    pub fn finalize(&mut self) -> anyhow::Result<serde_json::Value> {
        if self.media_id == "" {
            return Err((anyhow::anyhow!("media ID not set, call Init first")).into());
        }
        if self.verbose {
            print!("\033[32mFinalizing media upload...\033[0m\n");
        }
        let mut final_url = media_endpoint + format!("/{}/finalize", self.media_id);
        let mut request_options = RequestOptions {
            method: "POST".to_string(),
            endpoint: final_url,
            headers: self.headers,
            data: "".to_string(),
            auth_type: self.auth_type,
            username: self.username,
            verbose: self.verbose,
            trace: self.trace,
            ..Default::default()
        };
        let (response, client_err) = self.client.send_request(request_options);
        if client_err.is_some() {
            return Err(
                (anyhow::anyhow!("finalize request failed: {}", client_err)).into(),
            );
        }
        Ok(response)
    }
    /// CheckStatus checks the status of the media upload
    pub fn check_status(&mut self) -> anyhow::Result<serde_json::Value> {
        if self.media_id == "" {
            return Err((anyhow::anyhow!("media ID not set, call Init first")).into());
        }
        if self.verbose {
            println!("{:?}", "Checking media status...");
        }
        let mut url = MediaEndpoint + "?command=STATUS&media_id=" + self.media_id;
        let mut request_options = RequestOptions {
            method: "GET".to_string(),
            endpoint: url,
            headers: vec![],
            data: "".to_string(),
            auth_type: self.auth_type,
            username: self.username,
            verbose: self.verbose,
            trace: self.trace,
            ..Default::default()
        };
        let (response, client_err) = self.client.send_request(request_options);
        if client_err.is_some() {
            return Err(
                (anyhow::anyhow!("status request failed: {}", client_err)).into(),
            );
        }
        if self.verbose {
            utils.format_and_print_response(response);
        }
        Ok(response)
    }
    /// WaitForProcessing waits for media processing to complete
    pub fn wait_for_processing(&mut self) -> anyhow::Result<serde_json::Value> {
        if self.media_id == "" {
            return Err((anyhow::anyhow!("media ID not set, call Init first")).into());
        }
        if self.verbose {
            print!("\033[32mWaiting for media processing to complete...\033[0m\n");
        }
        loop {
            let mut response = self.check_status()?;
            let mut status_response: struct_Data_struct_ProcessingInfo_struct_State_string__json___state_____CheckAfterSecs_int__json___check_after_secs_____ProgressPercent_int__json___progress_percent______json___processing_info______json___data____ = Default::default();
            let mut err = serde_json::from_str(&response);
            let mut state = status_response.data.processing_info.state;
            if state == "succeeded" {
                if self.verbose {
                    print!("\033[32mMedia processing complete!\033[0m\n");
                }
                Ok(response)
            } else if state == "failed" {
                return Err((anyhow::anyhow!("media processing failed")).into());
            }
            let mut check_after_secs = status_response
                .data
                .processing_info
                .check_after_secs;
            if check_after_secs <= 0 {
                check_after_secs = 1;
            }
            if self.verbose {
                print!(
                    "\033[33mMedia processing in progress ({}%%), checking again in {} seconds...\033[0m\n",
                    status_response.data.processing_info.progress_percent,
                    check_after_secs
                );
            }
            std::thread::sleep(time.duration(check_after_secs) * time.second);
        }
    }
    /// GetMediaID returns the media ID
    pub fn get_media_id(&mut self) -> String {
        self.media_id
    }
    /// SetMediaID sets the media ID
    pub fn set_media_id(&mut self, media_id: &str) {
        self.media_id = media_id;
    }
}
