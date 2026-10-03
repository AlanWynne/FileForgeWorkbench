//! Per-entry value-widget rendering for the Config View
//! (configuration-system Requirement 15.3). Split out of
//! `config_panel/render.rs` for the 400-line rule; behaviour unchanged.

use eframe::egui;
use ff_config::schema::SchemaEntry;
use ff_config::value::ConfigValue;
use ff_config::ConfigHandle;

use super::commit::commit_value;
use super::ConfigPanelState;

/// Render the appropriate input widget for a schema entry's value type.
///
/// Validates: Requirement 15.3
pub(super) fn render_widget(
    ui: &mut egui::Ui,
    state: &mut ConfigPanelState,
    config: &ConfigHandle,
    entry: &SchemaEntry,
    effective: &ConfigValue,
) -> Option<egui::Id> {
    use ff_config::error::ValueType;

    let key = entry.key.clone();

    match entry.value_type {
        // Boolean -> checkbox -- Req 15.3
        ValueType::Boolean => {
            let mut checked = matches!(effective, ConfigValue::Boolean(true));
            let resp = ui.checkbox(&mut checked, "");
            if resp.changed() {
                let new_val = ConfigValue::Boolean(checked);
                commit_value(state, config, &key, new_val);
            }
            Some(resp.id)
        }

        // Integer with min+max -> slider; without -> text field -- Req 15.3
        ValueType::Integer => {
            let current = match effective {
                ConfigValue::Integer(i) => *i,
                _ => 0,
            };
            if let Some(ref c) = entry.constraints {
                if let (Some(min), Some(max)) = (c.min, c.max) {
                    let mut val = current;
                    let resp = ui.add(egui::Slider::new(&mut val, min as i64..=max as i64));
                    if resp.changed() {
                        commit_value(state, config, &key, ConfigValue::Integer(val));
                    }
                    return Some(resp.id);
                }
            }
            // Numeric text field
            if !state.pending.contains_key(&key) {
                state.pending.insert(key.clone(), current.to_string());
            }
            let pending = state.pending.get_mut(&key).unwrap();
            let resp = ui.text_edit_singleline(pending);
            if resp.lost_focus() {
                let text = pending.clone();
                match text.trim().parse::<i64>() {
                    Ok(v) => {
                        state.errors.remove(&key);
                        commit_value(state, config, &key, ConfigValue::Integer(v));
                        state.pending.remove(&key);
                    }
                    Err(_) => {
                        state
                            .errors
                            .insert(key.clone(), "Must be a whole number".to_string());
                    }
                }
            } else if !resp.has_focus() {
                *pending = current.to_string();
            }
            Some(resp.id)
        }

        // Float with min+max -> slider; without -> text field -- Req 15.3
        ValueType::Float => {
            let current = match effective {
                ConfigValue::Float(f) => *f,
                _ => 0.0,
            };
            if let Some(ref c) = entry.constraints {
                if let (Some(min), Some(max)) = (c.min, c.max) {
                    let mut val = current;
                    let resp = ui.add(egui::Slider::new(&mut val, min..=max).step_by(0.1));
                    if resp.changed() {
                        commit_value(state, config, &key, ConfigValue::Float(val));
                    }
                    return Some(resp.id);
                }
            }
            if !state.pending.contains_key(&key) {
                state.pending.insert(key.clone(), current.to_string());
            }
            let pending = state.pending.get_mut(&key).unwrap();
            let resp = ui.text_edit_singleline(pending);
            if resp.lost_focus() {
                let text = pending.clone();
                match text.trim().parse::<f64>() {
                    Ok(v) => {
                        state.errors.remove(&key);
                        commit_value(state, config, &key, ConfigValue::Float(v));
                        state.pending.remove(&key);
                    }
                    Err(_) => {
                        state
                            .errors
                            .insert(key.clone(), "Must be a number".to_string());
                    }
                }
            } else if !resp.has_focus() {
                *pending = current.to_string();
            }
            Some(resp.id)
        }

        // String with allowed_values -> combo box; without -> text field -- Req 15.3
        ValueType::String => {
            let current = match effective {
                ConfigValue::String(s) => s.clone(),
                _ => String::new(),
            };
            if let Some(ref c) = entry.constraints {
                if let Some(ref allowed) = c.allowed_values {
                    let options: Vec<String> = allowed
                        .iter()
                        .filter_map(|v| {
                            if let ConfigValue::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !options.is_empty() {
                        let mut selected = current.clone();
                        let combo = egui::ComboBox::from_id_salt(&key)
                            .selected_text(&selected)
                            .show_ui(ui, |ui| {
                                for opt in &options {
                                    ui.selectable_value(&mut selected, opt.clone(), opt.as_str());
                                }
                            });
                        if selected != current {
                            commit_value(state, config, &key, ConfigValue::String(selected));
                        }
                        return Some(combo.response.id);
                    }
                }
            }
            // Plain text field -- only cache in pending while the field has focus.
            // If there is no in-progress edit, always seed from the live effective value
            // so the default is always visible even after a hot-reload or first open.
            if !state.pending.contains_key(&key) {
                state.pending.insert(key.clone(), current.clone());
            }
            let pending = state.pending.get_mut(&key).unwrap();
            // Path keys get full available width; other string keys get 400 px.
            let is_path = key.contains("root") || key.contains("dir") || key.contains("path");
            let desired_width = if is_path { f32::INFINITY } else { 400.0 };
            let resp = ui.add(egui::TextEdit::singleline(pending).desired_width(desired_width));
            if resp.lost_focus() {
                let text = pending.clone();
                state.errors.remove(&key);
                commit_value(state, config, &key, ConfigValue::String(text));
                // After commit, re-seed from the now-effective value next frame.
                state.pending.remove(&key);
            } else if !resp.has_focus() {
                // Not focused and no pending edit -- keep in sync with effective value.
                *pending = current.clone();
            }
            Some(resp.id)
        }

        // Array / Table -- read-only display for now
        ValueType::Array | ValueType::Table => {
            ui.label(egui::RichText::new("[complex value -- edit TOML file directly]").weak());
            None
        }
    }
}
