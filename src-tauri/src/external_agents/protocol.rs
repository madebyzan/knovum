use agent_client_protocol::schema::{
    v1::{Implementation, InitializeRequest},
    ProtocolVersion,
};

pub(super) fn initialize_request() -> InitializeRequest {
    InitializeRequest::new(ProtocolVersion::V1)
        .client_info(Implementation::new("knovum", env!("CARGO_PKG_VERSION")).title("Knovum"))
}
