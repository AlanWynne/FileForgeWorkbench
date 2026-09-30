use std::path::Path;

/// Export pre-rendered HTML body to a standalone HTML file.
pub fn export_html_file(body_html: &str, title: &str, out_path: &Path) -> anyhow::Result<()> {
    let html = build_standalone_html(title, body_html);
    std::fs::write(out_path, html)?;
    Ok(())
}

/// Build a complete standalone HTML document from a body fragment and title.
pub fn build_standalone_html(title: &str, body_html: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>{title}</title>
<style>
body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;
  padding:40px;line-height:1.7;color:#1a1a1a;max-width:800px;margin:0 auto;}}
h1{{font-size:1.8em;border-bottom:1px solid #ddd;padding-bottom:8px;}}
h2{{font-size:1.4em;color:#333;margin-top:24px;}}
h3{{font-size:1.2em;color:#444;}}
code{{background:#f4f4f4;padding:2px 5px;border-radius:3px;
  font-family:Consolas,monospace;font-size:.9em;}}
pre{{background:#f8f8f8;border:1px solid #ddd;border-radius:6px;
  padding:12px 16px;overflow-x:auto;}}
pre code{{background:none;padding:0;}}
blockquote{{border-left:3px solid #ccc;padding-left:12px;margin-left:0;color:#555;}}
table{{width:100%;border-collapse:collapse;margin:12px 0;}}
th,td{{border:1px solid #ddd;padding:8px 12px;text-align:left;}}
th{{background:#f4f4f4;font-weight:600;}}
img{{max-width:100%;}}
a{{color:#0066cc;}}
@media print{{body{{padding:0;}}}}
</style>
</head>
<body>
{body_html}
</body>
</html>"#
    )
}
