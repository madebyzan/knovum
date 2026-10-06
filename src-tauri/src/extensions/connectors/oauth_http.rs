use futures::StreamExt;
use rmcp::transport::auth::{
    OAuthHttpClient, OAuthHttpClientFuture, OAuthHttpRedirectPolicy, OAuthHttpRequest,
};
use std::{sync::Arc, time::Duration};

pub(super) struct ConnectorOAuthHttp {
    pub allow_local: bool,
}

impl OAuthHttpClient for ConnectorOAuthHttp {
    fn execute(&self, operation: OAuthHttpRequest) -> OAuthHttpClientFuture<'_> {
        Box::pin(async move {
            let duration = operation.timeout.unwrap_or(Duration::from_secs(30));
            tokio::time::timeout(duration, self.request(operation))
                .await
                .map_err(std::io::Error::other)?
        })
    }
}

impl ConnectorOAuthHttp {
    async fn request(
        &self,
        operation: OAuthHttpRequest,
    ) -> Result<oauth2::HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let (parts, body) = operation.request.into_parts();
        let mut url = url::Url::parse(&parts.uri.to_string())?;
        let origin = url.origin();
        for redirects in 0..=5 {
            let client = super::super::mcp::endpoint::http_client(url.as_str(), self.allow_local)
                .await
                .map_err(std::io::Error::other)?;
            let response = client
                .request(parts.method.clone(), url.as_str())
                .headers(parts.headers.clone())
                .body(body.clone())
                .send()
                .await?;
            if response.status().is_redirection()
                && matches!(operation.redirect_policy, OAuthHttpRedirectPolicy::Follow)
            {
                if redirects == 5 {
                    return Err(std::io::Error::other("OAuth discovery redirect limit").into());
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .ok_or_else(|| std::io::Error::other("OAuth redirect location missing"))?
                    .to_str()?;
                let next = url.join(location)?;
                if next.origin() != origin
                    || !next.username().is_empty()
                    || next.password().is_some()
                {
                    return Err(std::io::Error::other(
                        "OAuth discovery cross-origin redirect rejected",
                    )
                    .into());
                }
                url = next;
                continue;
            }
            let mut builder = tauri::http::Response::builder().status(response.status());
            *builder.headers_mut().expect("response builder headers") = response.headers().clone();
            let mut bytes = Vec::new();
            let mut stream = response.bytes_stream();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                if bytes.len() + chunk.len() > 1024 * 1024 {
                    return Err(std::io::Error::other("OAuth response exceeds 1 MiB").into());
                }
                bytes.extend_from_slice(&chunk);
            }
            return Ok(builder.body(bytes)?);
        }
        unreachable!("bounded redirect loop returns")
    }
}

pub(super) fn client(allow_local: bool) -> Arc<dyn OAuthHttpClient> {
    Arc::new(ConnectorOAuthHttp { allow_local })
}
