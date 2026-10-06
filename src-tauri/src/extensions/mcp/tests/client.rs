use super::*;
use rmcp::{model::*, RoleServer, ServerHandler};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
struct ProtocolFixture;
impl ServerHandler for ProtocolFixture {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .enable_prompts()
            .build();
        info
    }
    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        let second = request.and_then(|request| request.cursor).is_some();
        let schema = json!({"type":"object","properties":{},"additionalProperties":false});
        let mut page = ListToolsResult::with_all_items(vec![Tool::new(
            if second { "second" } else { "first" },
            "Fixture tool",
            Arc::new(schema.as_object().unwrap().clone()),
        )]);
        if !second {
            page.next_cursor = Some("page-2".into());
        }
        Ok(page)
    }
    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        let second = request.and_then(|request| request.cursor).is_some();
        let mut page=ListResourcesResult::with_all_items(vec![serde_json::from_value(json!({"uri":if second {"fixture://second"}else{"fixture://document"},"name":"Document","mimeType":"text/plain"})).unwrap()]);
        if !second {
            page.next_cursor = Some("page-2".into());
        }
        Ok(page)
    }
    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, rmcp::ErrorData> {
        let second = request.and_then(|request| request.cursor).is_some();
        let mut page=ListResourceTemplatesResult::with_all_items(vec![serde_json::from_value(json!({"uriTemplate":if second {"fixture://second/{id}"}else{"fixture://document/{id}"},"name":"Document by ID"})).unwrap()]);
        if !second {
            page.next_cursor = Some("page-2".into());
        }
        Ok(page)
    }
    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::ErrorData> {
        let second = request.and_then(|request| request.cursor).is_some();
        let mut page = ListPromptsResult::with_all_items(vec![serde_json::from_value(
            json!({"name":if second {"second"}else{"draft"},"description":"Draft reference"}),
        )
        .unwrap()]);
        if !second {
            page.next_cursor = Some("page-2".into());
        }
        Ok(page)
    }
    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, rmcp::ErrorData> {
        Ok(ReadResourceResponse::Complete(serde_json::from_value(json!({"contents":[{"uri":request.uri,"text":"original text","mimeType":"text/plain"}]})).unwrap()))
    }
    async fn get_prompt(
        &self,
        _request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, rmcp::ErrorData> {
        Ok(GetPromptResponse::Complete(serde_json::from_value(json!({"messages":[{"role":"user","content":{"type":"text","text":"external reference"}}]})).unwrap()))
    }
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, rmcp::ErrorData> {
        if request.name == "first" {
            let elicitation=serde_json::from_value(json!({"mode":"form","message":"Choose a label","requestedSchema":{"type":"object","properties":{"label":{"type":"string"}},"required":["label"]}})).unwrap();
            let answer = context
                .peer
                .create_elicitation(elicitation)
                .await
                .map_err(|error| rmcp::ErrorData::internal_error(error.to_string(), None))?;
            assert_eq!(answer.action, ElicitationAction::Accept);
            context
                .peer
                .notify_progress(
                    serde_json::from_value(
                        json!({"progressToken":"fixture-operation","progress":1,"total":1}),
                    )
                    .unwrap(),
                )
                .await
                .map_err(|error| rmcp::ErrorData::internal_error(error.to_string(), None))?;
            Ok(CallToolResponse::Complete(CallToolResult::success(vec![
                ContentBlock::text("actual receipt"),
            ])))
        } else {
            Ok(CallToolResponse::Complete(CallToolResult::error(vec![
                ContentBlock::text("actual failure"),
            ])))
        }
    }
}
#[derive(Default)]
struct UserInteraction {
    forms: AtomicUsize,
    progress: AtomicUsize,
}
impl McpInteraction for UserInteraction {
    fn elicit(
        &self,
        _request: ElicitRequestParams,
        _cancellation: CancellationToken,
    ) -> BoxFuture<'_, Result<ElicitResult, String>> {
        Box::pin(async move {
            self.forms.fetch_add(1, Ordering::Relaxed);
            let mut answer = ElicitResult::new(ElicitationAction::Accept);
            answer.content = Some(json!({"label":"reviewed"}));
            Ok(answer)
        })
    }
    fn progress(&self, _progress: ProgressNotificationParam) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.progress.fetch_add(1, Ordering::Relaxed);
        })
    }
}
#[tokio::test]
async fn http_json_and_sse_use_the_same_official_catalog_and_interaction_contracts() {
    use rmcp::transport::{
        streamable_http_client::StreamableHttpClientTransportConfig,
        streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        },
        StreamableHttpClientTransport,
    };
    for json_response in [true, false] {
        let stop = CancellationToken::new();
        let service: StreamableHttpService<ProtocolFixture, LocalSessionManager> =
            StreamableHttpService::new(
                || Ok(ProtocolFixture),
                Default::default(),
                StreamableHttpServerConfig::default()
                    .with_json_response(json_response)
                    .with_cancellation_token(stop.child_token()),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = axum::Router::new().nest_service("/mcp", service);
        let shutdown = stop.clone();
        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown.cancelled_owned())
                .await
                .unwrap();
        });
        let uri = format!("http://{address}/mcp");
        let http = super::super::endpoint::http_client(&uri, true)
            .await
            .unwrap();
        let interaction = Arc::new(UserInteraction::default());
        let handler = ExtensionClient {
            interaction: Some(interaction.clone()),
        };
        let mut client = handler
            .serve(StreamableHttpClientTransport::with_client(
                super::super::http_client::BoundedHttpClient(http),
                StreamableHttpClientTransportConfig::with_uri(uri).reinit_on_expired_session(false),
            ))
            .await
            .unwrap();
        let catalog = super::super::catalog::read(&client).await.unwrap();
        assert_eq!(catalog.tools.len(), 2);
        assert_eq!(catalog.resources.len(), 2);
        assert_eq!(catalog.resource_templates.len(), 2);
        assert_eq!(catalog.prompts.len(), 2);
        let receipt = client
            .call_tool(CallToolRequestParams::new("first"))
            .await
            .unwrap();
        assert_ne!(receipt.is_error, Some(true));
        assert_eq!(interaction.forms.load(Ordering::Relaxed), 1);
        let failure = client
            .call_tool(CallToolRequestParams::new("second"))
            .await
            .unwrap();
        assert_eq!(failure.is_error, Some(true));
        let resource = client
            .read_resource(ReadResourceRequestParams::new("fixture://document"))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(resource).unwrap()["contents"][0]["text"],
            "original text"
        );
        let prompt = client
            .get_prompt(GetPromptRequestParams::new("draft"))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(prompt).unwrap()["messages"][0]["content"]["text"],
            "external reference"
        );
        client
            .close_with_timeout(Duration::from_secs(2))
            .await
            .unwrap();
        stop.cancel();
        server.await.unwrap();
        assert_eq!(interaction.progress.load(Ordering::Relaxed), 1);
    }
}
