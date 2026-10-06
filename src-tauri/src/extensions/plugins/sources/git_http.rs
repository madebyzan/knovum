use gix::protocol::transport::client::blocking_io::http::{
    self, GetResponse, Http, PostBodyDataKind, PostResponse,
};
use gix::{error::ResultExt, ExnMessageResult, ExnResult};
use std::{
    io::{self, BufRead, BufReader, Cursor, Read, Write},
    sync::{Arc, Mutex},
};

pub(super) struct GitHttp {
    pub client: reqwest::blocking::Client,
    pub origin: url::Url,
}
type Reader = Box<dyn BufRead + Send>;
const MAX_REQUEST: usize = 1024 * 1024;
const MAX_RESPONSE: u64 = 256 * 1024 * 1024;

impl GitHttp {
    fn request(
        &self,
        url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
        method: reqwest::Method,
    ) -> io::Result<reqwest::blocking::RequestBuilder> {
        let target = url::Url::parse(url).map_err(io::Error::other)?;
        if target.origin() != self.origin.origin()
            || !target.username().is_empty()
            || target.password().is_some()
        {
            return Err(io::Error::other("git-source-origin-changed"));
        }
        let mut request = self.client.request(method, target);
        for header in headers {
            let (name, value) = header
                .as_ref()
                .split_once(':')
                .ok_or_else(|| io::Error::other("git-header-invalid"))?;
            request = request.header(name.trim(), value.trim());
        }
        Ok(request)
    }
}

impl Http for GitHttp {
    type Headers = Reader;
    type ResponseBody = Reader;
    type PostBody = GitUpload;
    fn get(
        &mut self,
        url: &str,
        _base_url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> ExnMessageResult<GetResponse<Reader, Reader>> {
        let response = self
            .request(url, headers, reqwest::Method::GET)
            .and_then(send)
            .or_raise(|| gix::error::message("Git HTTPS discovery failed"))?;
        let headers = header_bytes(&response);
        Ok(GetResponse {
            headers: Box::new(Cursor::new(headers)),
            body: Box::new(BufReader::new(BoundedResponse {
                response,
                remaining: MAX_RESPONSE,
            })),
        })
    }
    fn post(
        &mut self,
        url: &str,
        _base_url: &str,
        headers: impl IntoIterator<Item = impl AsRef<str>>,
        _body: PostBodyDataKind,
    ) -> ExnMessageResult<PostResponse<Reader, Reader, GitUpload>> {
        let request = self
            .request(url, headers, reqwest::Method::POST)
            .or_raise(|| gix::error::message("Git HTTPS fetch request failed"))?;
        let shared = Arc::new(Mutex::new(PendingResponse {
            request: Some(request),
            upload: Vec::new(),
            ready: false,
            response: None,
            headers: Vec::new(),
        }));
        Ok(PostResponse {
            post_body: GitUpload(shared.clone()),
            headers: Box::new(BufReader::new(GitResponse {
                shared: shared.clone(),
                headers: Some(Cursor::new(Vec::new())),
                initialized: false,
            })),
            body: Box::new(BufReader::new(GitResponse {
                shared,
                headers: None,
                initialized: false,
            })),
        })
    }
    fn configure(&mut self, config: &dyn std::any::Any) -> ExnResult {
        if !config.is::<http::Options>() {
            return Err(gix::error::Exn::new(gix::error::message(
                "Git transport configuration is invalid",
            ))
            .erased());
        }
        Ok(())
    }
}

fn send(request: reqwest::blocking::RequestBuilder) -> io::Result<reqwest::blocking::Response> {
    let response = request.send().map_err(io::Error::other)?;
    if !response.status().is_success() {
        return Err(io::Error::other("git-source-http-status-rejected"));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE)
    {
        return Err(io::Error::other("git-source-response-size-limit"));
    }
    Ok(response)
}
fn header_bytes(response: &reqwest::blocking::Response) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (name, value) in response.headers() {
        bytes.extend_from_slice(name.as_str().as_bytes());
        bytes.extend_from_slice(b": ");
        bytes.extend_from_slice(value.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

struct BoundedResponse {
    response: reqwest::blocking::Response,
    remaining: u64,
}
impl Read for BoundedResponse {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.remaining == 0 {
            let mut extra = [0];
            return if self.response.read(&mut extra)? == 0 {
                Ok(0)
            } else {
                Err(io::Error::other("git-source-response-size-limit"))
            };
        }
        let limit = bytes.len().min(self.remaining as usize);
        let count = self.response.read(&mut bytes[..limit])?;
        self.remaining -= count as u64;
        Ok(count)
    }
}
struct PendingResponse {
    request: Option<reqwest::blocking::RequestBuilder>,
    upload: Vec<u8>,
    ready: bool,
    response: Option<BoundedResponse>,
    headers: Vec<u8>,
}
pub(super) struct GitUpload(Arc<Mutex<PendingResponse>>);
impl Write for GitUpload {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut pending = self
            .0
            .lock()
            .map_err(|_| io::Error::other("git-upload-state-unavailable"))?;
        if pending.upload.len() + bytes.len() > MAX_REQUEST {
            return Err(io::Error::other("git-fetch-request-size-limit"));
        }
        pending.upload.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Drop for GitUpload {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.0.lock() {
            pending.ready = true;
        }
    }
}
struct GitResponse {
    shared: Arc<Mutex<PendingResponse>>,
    headers: Option<Cursor<Vec<u8>>>,
    initialized: bool,
}
impl Read for GitResponse {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let mut pending = self
            .shared
            .lock()
            .map_err(|_| io::Error::other("git-response-state-unavailable"))?;
        if pending.response.is_none() {
            if !pending.ready {
                return Err(io::Error::other("git-upload-must-close-before-response"));
            }
            let request = pending
                .request
                .take()
                .ok_or_else(|| io::Error::other("git-request-already-consumed"))?;
            let body = std::mem::take(&mut pending.upload);
            let response = send(request.body(body))?;
            pending.headers = header_bytes(&response);
            pending.response = Some(BoundedResponse {
                response,
                remaining: MAX_RESPONSE,
            });
        }
        if let Some(headers) = &mut self.headers {
            if !self.initialized {
                *headers = Cursor::new(std::mem::take(&mut pending.headers));
                self.initialized = true;
            }
            headers.read(bytes)
        } else {
            pending
                .response
                .as_mut()
                .expect("received response")
                .read(bytes)
        }
    }
}
