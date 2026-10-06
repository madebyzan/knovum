use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use reqwest::header::{HeaderName, HeaderValue};
use rmcp::{
    model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
    transport::streamable_http_client::{
        AuthRequiredError, InsufficientScopeError, SseError, StreamableHttpClient,
        StreamableHttpError, StreamableHttpPostResponse,
    },
};
use std::{collections::HashMap, sync::Arc};

const RESPONSE_LIMIT: usize = 8 * 1024 * 1024;

/// Applies Knovum's raw response budget at the SDK HTTP backend boundary.
#[derive(Clone)]
pub(crate) struct BoundedHttpClient(pub(crate) reqwest::Client);

impl StreamableHttpClient for BoundedHttpClient {
    type Error = reqwest::Error;
    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, StreamableHttpError<Self::Error>> {
        self.post(
            uri,
            message,
            session_id,
            auth_header,
            custom_headers,
            RESPONSE_LIMIT,
        )
        .await
    }
    async fn post_message_with_max_sse_event_size(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
        max_size: usize,
    ) -> Result<StreamableHttpPostResponse, StreamableHttpError<Self::Error>> {
        self.post(
            uri,
            message,
            session_id,
            auth_header,
            custom_headers,
            max_size.min(RESPONSE_LIMIT),
        )
        .await
    }
    async fn get_stream(
        &self,
        uri: Arc<str>,
        session_id: Option<Arc<str>>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<
        BoxStream<'static, Result<sse_stream::Sse, SseError>>,
        StreamableHttpError<Self::Error>,
    > {
        self.0
            .get_stream_with_max_sse_event_size(
                uri,
                session_id,
                last_event_id,
                auth_header,
                custom_headers,
                RESPONSE_LIMIT,
            )
            .await
    }
    async fn get_stream_with_max_sse_event_size(
        &self,
        uri: Arc<str>,
        session_id: Option<Arc<str>>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
        max_size: usize,
    ) -> Result<
        BoxStream<'static, Result<sse_stream::Sse, SseError>>,
        StreamableHttpError<Self::Error>,
    > {
        self.0
            .get_stream_with_max_sse_event_size(
                uri,
                session_id,
                last_event_id,
                auth_header,
                custom_headers,
                max_size.min(RESPONSE_LIMIT),
            )
            .await
    }
    async fn delete_session(
        &self,
        uri: Arc<str>,
        session_id: Arc<str>,
        auth_header: Option<String>,
        custom_headers: HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), StreamableHttpError<Self::Error>> {
        self.0
            .delete_session(uri, session_id, auth_header, custom_headers)
            .await
    }
}

impl BoundedHttpClient {
    async fn post(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        headers: HashMap<HeaderName, HeaderValue>,
        limit: usize,
    ) -> Result<StreamableHttpPostResponse, StreamableHttpError<reqwest::Error>> {
        use rmcp::transport::common::http_header::{
            EVENT_STREAM_MIME_TYPE, HEADER_SESSION_ID, JSON_MIME_TYPE,
        };
        let mut request = self.0.post(uri.as_ref());
        for (name, value) in headers {
            if matches!(name.as_str(), "accept" | "mcp-session-id" | "last-event-id") {
                return Err(StreamableHttpError::ReservedHeaderConflict(
                    name.to_string(),
                ));
            }
            request = request.header(name, value);
        }
        request = request.header(
            reqwest::header::ACCEPT,
            format!("{EVENT_STREAM_MIME_TYPE}, {JSON_MIME_TYPE}"),
        );
        if let Some(token) = auth_header {
            request = request.bearer_auth(token);
        }
        let attached = session_id.is_some();
        if let Some(session_id) = session_id {
            request = request.header(HEADER_SESSION_ID, session_id.as_ref());
        }
        let response = request
            .json(&message)
            .send()
            .await
            .map_err(StreamableHttpError::Client)?;
        let status = response.status();
        let challenge = response
            .headers()
            .get(reqwest::header::WWW_AUTHENTICATE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        match (status, challenge) {
            (reqwest::StatusCode::UNAUTHORIZED, Some(challenge)) => {
                return Err(StreamableHttpError::AuthRequired(AuthRequiredError::new(
                    challenge,
                )))
            }
            (reqwest::StatusCode::FORBIDDEN, Some(challenge)) => {
                return Err(StreamableHttpError::InsufficientScope(
                    InsufficientScopeError::new(challenge, None),
                ))
            }
            (reqwest::StatusCode::NOT_FOUND, _) if attached => {
                return Err(StreamableHttpError::SessionExpired)
            }
            (reqwest::StatusCode::ACCEPTED | reqwest::StatusCode::NO_CONTENT, _) => {
                return Ok(StreamableHttpPostResponse::Accepted)
            }
            _ => {}
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let session = response
            .headers()
            .get(HEADER_SESSION_ID)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        if status.is_success()
            && response.content_length() == Some(0)
            && !matches!(message, ClientJsonRpcMessage::Request(_))
        {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        let stream = response.bytes_stream().scan(0usize, move |length, chunk| {
            let result = chunk.map_err(std::io::Error::other).and_then(|chunk| {
                *length += chunk.len();
                if *length > limit {
                    Err(std::io::Error::other("mcp-raw-response-budget-exceeded"))
                } else {
                    Ok(chunk)
                }
            });
            futures::future::ready(Some(result))
        });
        if status.is_success()
            && content_type
                .as_deref()
                .is_some_and(|value| value.starts_with(EVENT_STREAM_MIME_TYPE))
        {
            return Ok(StreamableHttpPostResponse::Sse(
                sse_stream::SseStream::from_bytes_stream(stream).boxed(),
                session,
            ));
        }
        if !content_type
            .as_deref()
            .is_some_and(|value| value.starts_with(JSON_MIME_TYPE))
        {
            return Err(StreamableHttpError::UnexpectedContentType(content_type));
        }
        let bytes = stream
            .try_fold(Vec::new(), |mut bytes, chunk| {
                bytes.extend_from_slice(&chunk);
                futures::future::ready(Ok(bytes))
            })
            .await
            .map_err(StreamableHttpError::Io)?;
        let message: ServerJsonRpcMessage = serde_json::from_slice(&bytes)?;
        if !status.is_success() && !matches!(message, ServerJsonRpcMessage::Error(_)) {
            return Err(StreamableHttpError::UnexpectedServerResponse(
                format!("MCP HTTP status {status}").into(),
            ));
        }
        Ok(StreamableHttpPostResponse::Json(message, session))
    }
}
