import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

if b'TabKind::SearchResults' in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ALREADY_DONE\n')
    sys.exit(0)

# Insert SearchResults arm before FileExplorerPanel arm
old = (
    b'                    TabKind::FileExplorerPanel => {' + sep +
    b'                        // Rendered above in the is_file_explorer block \xe2\x80\x94 unreachable here' + sep +
    b'                    }' + sep +
    b'                }' + sep +
    b'            });' + sep +
    b'        } // end !is_file_explorer'
)

new = (
    b'                    TabKind::SearchResults => {' + sep +
    b'                        // Validates: global-search Requirement 1.1, 4.1' + sep +
    b'                        let roots = collect_search_roots(&self.files_panel.registry, self.active_workspace.as_ref());' + sep +
    b'                        let outcome = crate::search_results_panel::render(' + sep +
    b'                            ui,' + sep +
    b'                            &mut self.search_results_panel,' + sep +
    b'                            &roots,' + sep +
    b'                            &self.runtime,' + sep +
    b'                        );' + sep +
    b'                        match outcome {' + sep +
    b'                            crate::search_results_panel::SearchPanelOutcome::OpenMatch { path, line } => {' + sep +
    b'                                if let Err(e) = self.tabs.open_file(&path, &self.runtime) {' + sep +
    b'                                    self.open_error = Some(e);' + sep +
    b'                                } else {' + sep +
    b'                                    // Scroll to the matching line.' + sep +
    b'                                    let idx = self.tabs.active_index();' + sep +
    b'                                    if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {' + sep +
    b'                                        tab.viewport.set_top_line(line.saturating_sub(1).max(1));' + sep +
    b'                                    }' + sep +
    b'                                }' + sep +
    b'                            }' + sep +
    b'                            crate::search_results_panel::SearchPanelOutcome::ReplaceAll => {' + sep +
    b'                                let unsaved: Vec<String> = self.tabs.tabs().iter()' + sep +
    b'                                    .filter(|t| t.is_modified)' + sep +
    b'                                    .filter_map(|t| t.path.clone())' + sep +
    b'                                    .collect();' + sep +
    b'                                let req = self.search_results_panel.build_request(roots).ok();' + sep +
    b'                                if let Some(r) = req {' + sep +
    b'                                    let results = self.search_results_panel.results.clone();' + sep +
    b'                                    match ff_global_search::GlobalReplaceEngine::replace_all(' + sep +
    b'                                        &results,' + sep +
    b'                                        &r,' + sep +
    b'                                        &self.search_results_panel.replace_text.clone(),' + sep +
    b'                                        &unsaved,' + sep +
    b'                                    ) {' + sep +
    b'                                        Ok((summary, _conflicts)) => {' + sep +
    b'                                            self.open_error = Some(format!(' + sep +
    b'                                                "Replaced {} occurrence(s) in {} file(s)",' + sep +
    b'                                                summary.replacements, summary.files_modified' + sep +
    b'                                            ));' + sep +
    b'                                        }' + sep +
    b'                                        Err(e) => {' + sep +
    b'                                            self.open_error = Some(format!("Replace failed: {e}"));' + sep +
    b'                                        }' + sep +
    b'                                    }' + sep +
    b'                                }' + sep +
    b'                            }' + sep +
    b'                            _ => {}' + sep +
    b'                        }' + sep +
    b'                    }' + sep +
    b'                    TabKind::FileExplorerPanel => {' + sep +
    b'                        // Rendered above in the is_file_explorer block \xe2\x80\x94 unreachable here' + sep +
    b'                    }' + sep +
    b'                }' + sep +
    b'            });' + sep +
    b'        } // end !is_file_explorer'
)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('old_in_data=' + str(old in data) + '\n')

if old not in data:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('ERROR: pattern not found\n')
    sys.exit(1)

data = data.replace(old, new, 1)
with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('WRITTEN OK\n')
