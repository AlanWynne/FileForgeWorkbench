import sys

LOG = r'c:\workspace\VSC\FileForgeWorkbench\tools\logs\cmd_edit.txt'
path = r'c:\workspace\VSC\FileForgeWorkbench\crates\ff-desktop\src\shell\render.rs'

with open(LOG, 'w', encoding='utf-8') as lf:
    lf.write('start\n')

with open(path, 'rb') as f:
    data = f.read()

sep = b'\r\n' if b'\r\n' in data else b'\n'

# 1. Add ff_global_search import after the existing imports block
if b'ff_global_search' not in data:
    old_import = b'use super::{FocusStop, WorkbenchShell};'
    new_import = b'use super::{FocusStop, WorkbenchShell};' + sep + b'use ff_global_search;'
    if old_import in data:
        data = data.replace(old_import, new_import, 1)
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('import added\n')
    else:
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('ERROR: import anchor not found\n')
else:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('import already present\n')

# 2. Add collect_search_roots helper before the open_mainframe_dsn helper
if b'fn collect_search_roots' not in data:
    old_helper = b'/// Resolve a Mainframe DSN to a physical path via the ff-desktop CatalogRegistry,'
    helper_fn = (
        b'/// Collect search root paths from the active workspace or mounted Native catalogs.' + sep +
        b'///' + sep +
        b'/// Validates: global-search Requirement 2.4' + sep +
        b'fn collect_search_roots(' + sep +
        b'    registry: &crate::catalog_registry::CatalogRegistry,' + sep +
        b'    workspace: Option<&ff_session::WorkspaceState>,' + sep +
        b') -> Vec<String> {' + sep +
        b'    if let Some(ws) = workspace {' + sep +
        b'        if !ws.roots.is_empty() {' + sep +
        b'            return ws.roots.iter().map(|p| p.to_string_lossy().into_owned()).collect();' + sep +
        b'        }' + sep +
        b'    }' + sep +
        b'    registry' + sep +
        b'        .list_by_type(crate::catalog_registry::CatalogType::Native)' + sep +
        b'        .iter()' + sep +
        b'        .map(|c| c.path.clone())' + sep +
        b'        .collect()' + sep +
        b'}' + sep +
        sep +
        b'/// Resolve a Mainframe DSN to a physical path via the ff-desktop CatalogRegistry,'
    )
    if old_helper in data:
        data = data.replace(old_helper, helper_fn, 1)
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('helper added\n')
    else:
        with open(LOG, 'a', encoding='utf-8') as lf:
            lf.write('ERROR: helper anchor not found\n')
else:
    with open(LOG, 'a', encoding='utf-8') as lf:
        lf.write('helper already present\n')

with open(path, 'wb') as f:
    f.write(data)

with open(LOG, 'a', encoding='utf-8') as lf:
    lf.write('DONE\n')
