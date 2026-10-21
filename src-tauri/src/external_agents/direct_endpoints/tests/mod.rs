use agent_client_protocol::schema::v1::{McpServer, McpServerHttp, McpServerStdio};

use super::environment;

fn knovum() -> McpServer {
    McpServer::Http(McpServerHttp::new("knovum", "http://127.0.0.1:43117/mcp"))
}

fn exemptions<'a>(environment: &'a [(&'static str, String)]) -> Vec<(&'static str, &'a str)> {
    environment
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect()
}

#[test]
fn exempts_every_endpoint_host_from_proxying() {
    let launch = environment(&[knovum()], |_| None).unwrap();

    assert_eq!(
        exemptions(&launch),
        [("NO_PROXY", "127.0.0.1"), ("no_proxy", "127.0.0.1")]
    );
}

#[test]
fn keeps_the_users_exemptions_from_either_spelling() {
    let launch = environment(&[knovum()], |name| match name {
        "NO_PROXY" => Some("corp.internal, localhost".into()),
        "no_proxy" => Some("localhost,127.0.0.1".into()),
        _ => None,
    })
    .unwrap();

    let merged = "corp.internal,localhost,127.0.0.1";
    assert_eq!(
        exemptions(&launch),
        [("NO_PROXY", merged), ("no_proxy", merged)]
    );
}

#[test]
fn leaves_the_environment_alone_without_network_endpoints() {
    let stdio = McpServer::Stdio(McpServerStdio::new("local", "/usr/bin/server"));

    assert!(environment(&[], |_| Some("corp.internal".into()))
        .unwrap()
        .is_empty());
    assert!(environment(&[stdio], |_| Some("corp.internal".into()))
        .unwrap()
        .is_empty());
}

#[test]
fn rejects_an_endpoint_without_a_host() {
    let server = McpServer::Http(McpServerHttp::new("broken", "unix:/tmp/knovum.sock"));

    assert!(environment(&[server], |_| None).is_err());
}
