path = r'crates\ff-desktop\src\shell\render.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Insert before the is_file_explorer block
anchor = '        let is_file_explorer = self.tabs.active_tab().kind == TabKind::FileExplorerPanel;'

insert = (
    '        // Validates: Requirement 20.1 -- Tab from CommandField in FilesPanel transfers focus\n'
    '        // to the first catalog node in the catalog tree.\n'
    '        let is_files_panel = self.tabs.active_tab().kind == TabKind::FilesPanel;\n'
    '        if is_files_panel && !self.modal_open && self.focus_stop == FocusStop::CommandField {\n'
    '            let tab_pressed = ctx.input_mut(|i| {\n'
    '                if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {\n'
    '                    i.events.retain(|e| {\n'
    '                        !matches!(e, egui::Event::Key { key: egui::Key::Tab, .. })\n'
    '                    });\n'
    '                    return true;\n'
    '                }\n'
    '                false\n'
    '            });\n'
    '            if tab_pressed {\n'
    '                self.files_panel.tree_focus_requested = true;\n'
    '            }\n'
    '        }\n'
    '\n'
)

if anchor in text:
    text = text.replace(anchor, insert + anchor, 1)
    print('Tab interception for FilesPanel inserted')
else:
    print('FAILED: anchor not found')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print('render.rs written.')
