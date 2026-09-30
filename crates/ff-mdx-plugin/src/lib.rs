mod viewer;

use std::sync::Arc;

use ff_plugin::{
    Capability, FileForgePlugin, PluginContext, PluginError, PluginMetadata, Version,
    ViewersCapability,
};

pub use viewer::MdxFileViewer;

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
        Ok(())
    }
    fn activate(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
