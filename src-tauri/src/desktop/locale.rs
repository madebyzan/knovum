use crate::preferences::LocalePreference;

pub(super) struct Labels {
    #[cfg(target_os = "macos")]
    pub menu: MenuLabels,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub tray: TrayLabels,
    pub completed: &'static str,
    pub failed: &'static str,
}

#[cfg(target_os = "macos")]
pub(super) struct MenuLabels {
    pub command_palette: &'static str,
    pub settings: &'static str,
    pub edit: &'static str,
    pub cut: &'static str,
    pub copy: &'static str,
    pub paste: &'static str,
    pub file: &'static str,
    pub new_space: &'static str,
    pub reopen_closed_tab: &'static str,
    pub close_tab: &'static str,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) struct TrayLabels {
    pub show: &'static str,
    pub quit: &'static str,
}

pub(super) fn labels(locale: LocalePreference) -> Result<Labels, String> {
    use LocalePreference::*;
    let [_command_palette, _settings, _edit, _cut, _copy, _paste, _file, _new_space, _reopen_closed_tab, _close_tab, _show, _quit, completed, failed] =
        match locale {
            System => {
                return Err(
                    "The system language must be resolved before updating desktop labels".into(),
                )
            }
            English => [
                "Command Palette…",
                "Settings…",
                "Edit",
                "Cut",
                "Copy",
                "Paste",
                "File",
                "New Space",
                "Reopen Closed Tab",
                "Close Tab",
                "Show Knovum",
                "Quit Knovum",
                "Task completed.",
                "Task failed. Open Task Center for details.",
            ],
            SimplifiedChinese => [
                "命令面板…",
                "设置…",
                "编辑",
                "剪切",
                "复制",
                "粘贴",
                "文件",
                "新建空间",
                "重新打开关闭的标签页",
                "关闭标签页",
                "显示知生知",
                "退出知生知",
                "任务已完成。",
                "任务失败。打开任务中心查看详情。",
            ],
            TraditionalChinese => [
                "命令面板…",
                "設定…",
                "編輯",
                "剪下",
                "複製",
                "貼上",
                "檔案",
                "新增空間",
                "重新開啟已關閉的分頁",
                "關閉分頁",
                "顯示知生知",
                "結束知生知",
                "任務已完成。",
                "任務失敗。開啟任務中心查看詳情。",
            ],
            Japanese => [
                "コマンドパレット…",
                "設定…",
                "編集",
                "切り取り",
                "コピー",
                "貼り付け",
                "ファイル",
                "新しいスペース",
                "閉じたタブを再度開く",
                "タブを閉じる",
                "Knovumを表示",
                "Knovumを終了",
                "タスクが完了しました。",
                "タスクが失敗しました。タスクセンターで詳細を確認してください。",
            ],
            Korean => [
                "명령 팔레트…",
                "설정…",
                "편집",
                "잘라내기",
                "복사",
                "붙여넣기",
                "파일",
                "새 공간",
                "닫은 탭 다시 열기",
                "탭 닫기",
                "Knovum 표시",
                "Knovum 종료",
                "작업이 완료되었습니다.",
                "작업이 실패했습니다. 작업 센터에서 자세한 내용을 확인하세요.",
            ],
            French => [
                "Palette de commandes…",
                "Réglages…",
                "Édition",
                "Couper",
                "Copier",
                "Coller",
                "Fichier",
                "Nouvel espace",
                "Rouvrir l’onglet fermé",
                "Fermer l’onglet",
                "Afficher Knovum",
                "Quitter Knovum",
                "Tâche terminée.",
                "Échec de la tâche. Consultez le centre des tâches pour plus de détails.",
            ],
            German => [
                "Befehlspalette…",
                "Einstellungen…",
                "Bearbeiten",
                "Ausschneiden",
                "Kopieren",
                "Einfügen",
                "Datei",
                "Neuer Wissensraum",
                "Geschlossenen Tab wieder öffnen",
                "Tab schließen",
                "Knovum anzeigen",
                "Knovum beenden",
                "Aufgabe abgeschlossen.",
                "Aufgabe fehlgeschlagen. Weitere Informationen finden Sie im Aufgabencenter.",
            ],
            Spanish => [
                "Paleta de comandos…",
                "Ajustes…",
                "Edición",
                "Cortar",
                "Copiar",
                "Pegar",
                "Archivo",
                "Nuevo espacio",
                "Reabrir pestaña cerrada",
                "Cerrar pestaña",
                "Mostrar Knovum",
                "Salir de Knovum",
                "Tarea completada.",
                "La tarea ha fallado. Consulta el centro de tareas para ver los detalles.",
            ],
            BrazilianPortuguese => [
                "Paleta de comandos…",
                "Configurações…",
                "Editar",
                "Recortar",
                "Copiar",
                "Colar",
                "Arquivo",
                "Novo espaço",
                "Reabrir aba fechada",
                "Fechar aba",
                "Mostrar Knovum",
                "Sair do Knovum",
                "Tarefa concluída.",
                "A tarefa falhou. Abra a central de tarefas para ver os detalhes.",
            ],
            Italian => [
                "Tavolozza dei comandi…",
                "Impostazioni…",
                "Modifica",
                "Taglia",
                "Copia",
                "Incolla",
                "File",
                "Nuovo spazio",
                "Riapri scheda chiusa",
                "Chiudi scheda",
                "Mostra Knovum",
                "Esci da Knovum",
                "Attività completata.",
                "L’attività non è riuscita. Apri il centro attività per i dettagli.",
            ],
            Russian => [
                "Палитра команд…",
                "Настройки…",
                "Правка",
                "Вырезать",
                "Копировать",
                "Вставить",
                "Файл",
                "Новое пространство",
                "Открыть закрытую вкладку",
                "Закрыть вкладку",
                "Показать Knovum",
                "Выйти из Knovum",
                "Задача завершена.",
                "Задача не выполнена. Подробности — в центре задач.",
            ],
        };
    Ok(Labels {
        #[cfg(target_os = "macos")]
        menu: MenuLabels {
            command_palette: _command_palette,
            settings: _settings,
            edit: _edit,
            cut: _cut,
            copy: _copy,
            paste: _paste,
            file: _file,
            new_space: _new_space,
            reopen_closed_tab: _reopen_closed_tab,
            close_tab: _close_tab,
        },
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        tray: TrayLabels {
            show: _show,
            quit: _quit,
        },
        completed,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_system_language_is_rejected() {
        assert!(labels(LocalePreference::System).is_err());
    }
}
