#[macro_use]
#[path = "../src/spaces/command_registry/declarations.rs"]
mod declarations;
macro_rules! command_name {
    ($name:ident) => { stringify!($name) };
    ($head:ident::$($rest:ident)::+) => { command_name!($($rest)::+) };
}
macro_rules! command_names {
    ($($($segments:ident)::+),* $(,)?) => { &[ $(command_name!($($segments)::+)),* ] };
}
pub(crate) fn commands() -> &'static [&'static str] {
    static COMMANDS: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
        let workspace: &[&str] = knovum_workspace_commands!(command_names);
        let knowledge: &[&str] = knovum_knowledge_commands!(command_names);
        let execution: &[&str] = knovum_execution_commands!(command_names);
        workspace
            .iter()
            .chain(knowledge)
            .chain(execution)
            .copied()
            .collect()
    });
    COMMANDS.as_slice()
}
