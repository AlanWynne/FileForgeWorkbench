import re

path = r'crates\ff-desktop\src\files_panel.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Step 1: replace 'for cat in visible {' with enumerated version
old1 = '                for cat in visible {'
new1 = '                for (idx, cat) in visible.iter().enumerate() {'
if old1 in text:
    text = text.replace(old1, new1, 1)
    print('Step 1 applied: for loop enumerated')
else:
    print('Step 1 FAILED: pattern not found')
    print(repr(text[text.find('for cat'):text.find('for cat')+50]))

# Step 2: insert focus block after selectable_label line (only the first occurrence inside render_section)
old2 = '                    let resp = ui.selectable_label(is_selected, label_text);\n'
new2 = (
    '                    let resp = ui.selectable_label(is_selected, label_text);\n'
    '                    // Validates: Requirement 20.1 -- Tab from command field focuses first catalog.\n'
    '                    if *focus_first && idx == 0 {\n'
    '                        *focus_first = false;\n'
    '                        let item_id = egui::Id::new("files_panel_cat").with(&cat.name);\n'
    '                        ui.ctx().memory_mut(|m| m.request_focus(item_id));\n'
    '                    }\n'
)
if old2 in text:
    text = text.replace(old2, new2, 1)
    print('Step 2 applied: focus block inserted')
else:
    print('Step 2 FAILED: selectable_label pattern not found')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print('File written.')
