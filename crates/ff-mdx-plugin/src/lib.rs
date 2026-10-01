//! FileForgeWorkbench plugin that contributes a Markdown `FileViewer`.
//!
//! The plugin advertises a Viewers capability and exposes [`MdxFileViewer`],
//! which renders `.md`/`.markdown` content to HTML via `ff-md-viewer`.

mod viewer;

use std::sync::Arc;

use ff_logging::{log, LogLevel};
use ff_plugin::{
    Capability, FileForgePlugin, PluginContext, PluginError, PluginMetadata, Version,
    ViewersCapability,
};

pub use viewer::MdxFileViewer;

/// The Markdown viewer plugin: advertises a Viewers capability and contributes
/// [`MdxFileViewer`] to the host's viewer registry.
pub struct MdxPlugin {
    metadata: PluginMetadata,
    capabilities: Vec<Capability>,
}

impl Default for MdxPlugin {
    fn default() -> Self {
        let version = Version::new(0, 1, 0);
        Self {
            capabilities: vec![Capability::Viewers(ViewersCapability {
                mime_types: vec!["text/markdown".to_string()],
                display_name: "Markdown Viewer".to_string(),
                version: version.clone(),
            })],
            metadata: PluginMetadata {
                name: "ff-mdx-plugin".to_string(),
                version,
                author: "FileForge Contributors".to_string(),
                description: "Renders .md files via pulldown-cmark".to_string(),
                dependencies: vec![],
                required_api_version: Version::new(0, 1, 0),
            },
        }
    }
}

impl FileForgePlugin for MdxPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn plugin_capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    fn initialize(&mut self, _ctx: Arc<PluginContext>) -> Result<(), PluginError> {
        log(LogLevel::Info, "ff-mdx-plugin", "initialize");
        Ok(())
    }
    fn activate(&mut self) -> Result<(), PluginError> {
        log(LogLevel::Info, "ff-mdx-plugin", "activate");
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), PluginError> {
        log(LogLevel::Info, "ff-mdx-plugin", "deactivate");
        Ok(())
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        log(LogLevel::Info, "ff-mdx-plugin", "shutdown");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::MdxPlugin;
    use ff_plugin::{Capability, FileForgePlugin};
    use ff_viewers::trait_def::FileViewer;

    #[test]
    fn plugin_metadata_matches_viewer_display_name_and_mime_types() {
        // Validates: Requirement 13.5
        let plugin = MdxPlugin::default();
        let viewer = super::MdxFileViewer;

        let Capability::Viewers(cap) = &plugin.plugin_capabilities()[0] else {
            panic!("first capability should be Viewers");
        };

        assert_eq!(cap.display_name, viewer.display_name());
        for mime in &cap.mime_types {
            assert!(
                viewer.supported_mime_types().contains(&mime.as_str()),
                "plugin-advertised MIME {mime} must be one the viewer supports"
            );
        }
    }
}
