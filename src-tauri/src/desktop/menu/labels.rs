pub struct Labels {
    pub command_palette: &'static str,
    pub settings: &'static str,
    pub file: &'static str,
    pub new_space: &'static str,
    pub reopen_closed_tab: &'static str,
    pub close_tab: &'static str,
}

pub fn labels(locale: &str) -> Labels {
    match locale {
        "zh-CN" => Labels {
            command_palette: "命令面板…",
            settings: "设置…",
            file: "文件",
            new_space: "新建空间",
            reopen_closed_tab: "重新打开关闭的标签页",
            close_tab: "关闭标签页",
        },
        _ => Labels {
            command_palette: "Command Palette…",
            settings: "Settings…",
            file: "File",
            new_space: "New Space",
            reopen_closed_tab: "Reopen Closed Tab",
            close_tab: "Close Tab",
        },
    }
}
