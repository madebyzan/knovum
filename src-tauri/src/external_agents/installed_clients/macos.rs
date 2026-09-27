use std::collections::HashSet;

use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSString, NSURL};

use crate::ai::{contracts::AgentKind, error::AiResult};

pub(crate) fn detect() -> AiResult<HashSet<AgentKind>> {
    // NSWorkspace is supplied by the desktop host's AppKit framework. objc2 owns
    // Objective-C reference counting; these are the documented NSWorkspace signatures.
    let workspace: Retained<AnyObject> = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
    let mut installed = HashSet::new();
    for agent in super::super::catalog::AGENTS {
        let Some(client) = &agent.desktop_client else {
            continue;
        };
        for identifier in client.bundle_identifiers {
            let identifier = NSString::from_str(identifier);
            let url: Option<Retained<NSURL>> = unsafe {
                msg_send![&*workspace, URLForApplicationWithBundleIdentifier: &*identifier]
            };
            let path = url.and_then(|url| url.path());
            if path.is_some_and(|path| std::path::Path::new(&path.to_string()).is_dir()) {
                installed.insert(agent.descriptor.kind);
                break;
            }
        }
    }
    Ok(installed)
}
