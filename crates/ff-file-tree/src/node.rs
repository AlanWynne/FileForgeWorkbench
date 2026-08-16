//! Core tree node types: NodeId, NodeType, FileCategory, TreeNode, TreeNodeData.

/// Opaque unique identifier for a tree node. Cheaply copyable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(pub(crate) u64);

impl NodeId {
    /// The root sentinel — parent of all top-level category nodes.
    pub const ROOT: Self = Self(0);
}

/// Discriminates the kind of resource a tree node represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NodeType {
    /// Top-level section header (Local Files, Catalogs, Connections).
    RootCategory,
    /// A bookmarked local filesystem root directory.
    BookmarkedRoot,
    /// A regular directory.
    Directory,
    /// A regular file.
    File,
    /// A symbolic link.
    SymbolicLink,
    /// A mounted dataset catalog root.
    CatalogRoot,
    /// A High-Level Qualifier grouping node.
    HlqGroup,
    /// A sequential dataset (DSORG=PS).
    DatasetSequential,
    /// A partitioned dataset (DSORG=PO).
    DatasetPartitioned,
    /// A PDS member.
    PdsMember,
    /// A Generation Data Group base.
    GdgBase,
    /// A GDG generation entry.
    GdgGeneration,
    /// A remote connection root (future).
    ConnectionRoot,
    /// Placeholder node ("No catalogs mounted", etc.).
    Placeholder,
    /// Loading indicator node.
    LoadingIndicator,
    /// Error indicator node.
    ErrorIndicator,
    /// Overflow indicator ("... and N more items").
    OverflowIndicator,
}

impl NodeType {
    /// Returns true if this node type can have children.
    pub fn is_expandable(self) -> bool {
        matches!(
            self,
            NodeType::RootCategory
                | NodeType::BookmarkedRoot
                | NodeType::Directory
                | NodeType::CatalogRoot
                | NodeType::HlqGroup
                | NodeType::DatasetPartitioned
                | NodeType::GdgBase
                | NodeType::ConnectionRoot
        )
    }

    /// Returns true if this node type is a leaf (cannot have children).
    pub fn is_leaf(self) -> bool {
        !self.is_expandable()
    }
}

/// Classification of files for colour-coding purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FileCategory {
    /// Binary or non-editable files.
    NonEditableBinary,
    /// Files with an associated FileForge structure definition.
    FileForgeStructured,
    /// Regular text files.
    StandardText,
    /// Unrecognised file type.
    Unknown,
    /// Directory nodes.
    Directory,
    /// Symbolic link nodes.
    SymbolicLink,
}

impl FileCategory {
    /// Returns the theme palette colour key for this category.
    pub fn colour_key(self) -> &'static str {
        match self {
            FileCategory::NonEditableBinary => "file_tree.non_editable_binary",
            FileCategory::FileForgeStructured => "file_tree.fileforge_structured",
            FileCategory::StandardText => "file_tree.standard_text",
            FileCategory::Unknown => "file_tree.unknown",
            FileCategory::Directory => "file_tree.directory",
            FileCategory::SymbolicLink => "file_tree.symbolic_link",
        }
    }

    /// Classify a file by its extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_ascii_lowercase().as_str() {
            "exe" | "dll" | "so" | "dylib" | "bin" | "obj" | "o" | "a" | "lib" | "png" | "jpg"
            | "jpeg" | "gif" | "bmp" | "ico" | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z"
            | "rar" | "pdf" | "doc" | "docx" | "xls" | "xlsx" => FileCategory::NonEditableBinary,
            "rs" | "c" | "cpp" | "cc" | "h" | "hpp" | "py" | "js" | "ts" | "java" | "go" | "rb"
            | "cs" | "kt" | "swift" | "lua" | "sh" | "bash" | "zsh" | "ps1" | "cob" | "cbl"
            | "jcl" | "rexx" | "asm" | "s" => FileCategory::StandardText,
            "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "cfg" | "conf" | "env"
            | "properties" | "plist" => FileCategory::StandardText,
            "txt" | "md" | "rst" | "log" | "csv" | "tsv" => FileCategory::StandardText,
            "" => FileCategory::Unknown,
            _ => FileCategory::Unknown,
        }
    }
}

/// A single node in the tree hierarchy.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Unique node identifier.
    pub id: NodeId,
    /// Parent node identifier (ROOT for top-level categories).
    pub parent: NodeId,
    /// Display label.
    pub label: String,
    /// The type of resource this node represents.
    pub node_type: NodeType,
    /// Whether this node is currently expanded.
    pub expanded: bool,
    /// Whether this node's children are currently being loaded.
    pub loading: bool,
    /// Ordered list of child node IDs.
    pub children: Vec<NodeId>,
    /// Whether children have been loaded at least once.
    pub children_loaded: bool,
    /// File size in bytes (for file nodes).
    pub size: Option<u64>,
    /// File category for colour coding.
    pub category: FileCategory,
    /// Whether this file has a FileForge structure definition.
    pub has_structure: bool,
    /// Whether this is a hidden file/directory.
    pub is_hidden: bool,
    /// Depth in the tree (0 = root categories).
    pub depth: u32,
}

impl TreeNode {
    /// Create a new tree node with the given parameters.
    pub fn new(
        id: NodeId,
        parent: NodeId,
        label: impl Into<String>,
        node_type: NodeType,
        depth: u32,
    ) -> Self {
        let label = label.into();
        let is_hidden = label.starts_with('.');
        let category = match node_type {
            NodeType::Directory
            | NodeType::BookmarkedRoot
            | NodeType::RootCategory
            | NodeType::HlqGroup => FileCategory::Directory,
            NodeType::SymbolicLink => FileCategory::SymbolicLink,
            NodeType::DatasetSequential
            | NodeType::DatasetPartitioned
            | NodeType::PdsMember
            | NodeType::GdgBase
            | NodeType::GdgGeneration => FileCategory::FileForgeStructured,
            NodeType::File => {
                let ext = label.rsplit('.').next().unwrap_or("");
                FileCategory::from_extension(ext)
            }
            _ => FileCategory::Unknown,
        };
        Self {
            id,
            parent,
            label,
            node_type,
            expanded: false,
            loading: false,
            children: Vec::new(),
            children_loaded: false,
            size: None,
            category,
            has_structure: false,
            is_hidden,
            depth,
        }
    }
}

/// Data for constructing a TreeNode from a VFS entry.
/// Used as the transfer type from async loader results.
#[derive(Debug, Clone)]
pub struct TreeNodeData {
    /// Display label.
    pub label: String,
    /// Node type.
    pub node_type: NodeType,
    /// File size.
    pub size: Option<u64>,
    /// File category.
    pub category: FileCategory,
    /// Whether the file has a structure definition.
    pub has_structure: bool,
    /// Whether hidden.
    pub is_hidden: bool,
}

impl TreeNodeData {
    /// Create a simple directory entry.
    pub fn directory(label: impl Into<String>) -> Self {
        let label = label.into();
        let is_hidden = label.starts_with('.');
        Self {
            label,
            node_type: NodeType::Directory,
            size: None,
            category: FileCategory::Directory,
            has_structure: false,
            is_hidden,
        }
    }

    /// Create a simple file entry.
    pub fn file(label: impl Into<String>) -> Self {
        let label = label.into();
        let is_hidden = label.starts_with('.');
        let ext = label.rsplit('.').next().unwrap_or("");
        let category = FileCategory::from_extension(ext);
        Self {
            label,
            node_type: NodeType::File,
            size: None,
            category,
            has_structure: false,
            is_hidden,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_root_is_zero() {
        // Validates: Requirement 2.1 — root sentinel identity
        assert_eq!(NodeId::ROOT, NodeId(0));
    }

    #[test]
    fn file_category_colour_keys_are_distinct() {
        // Validates: Requirement 4.5 — each category maps to a unique key
        let keys = [
            FileCategory::NonEditableBinary.colour_key(),
            FileCategory::FileForgeStructured.colour_key(),
            FileCategory::StandardText.colour_key(),
            FileCategory::Unknown.colour_key(),
            FileCategory::Directory.colour_key(),
            FileCategory::SymbolicLink.colour_key(),
        ];
        let unique: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(unique.len(), keys.len());
    }

    #[test]
    fn file_category_from_extension_binary() {
        // Validates: Requirement 4.5 — binary files classified correctly
        assert_eq!(
            FileCategory::from_extension("exe"),
            FileCategory::NonEditableBinary
        );
        assert_eq!(
            FileCategory::from_extension("png"),
            FileCategory::NonEditableBinary
        );
        assert_eq!(
            FileCategory::from_extension("zip"),
            FileCategory::NonEditableBinary
        );
    }

    #[test]
    fn file_category_from_extension_text() {
        // Validates: Requirement 4.5 — text files classified correctly
        assert_eq!(
            FileCategory::from_extension("rs"),
            FileCategory::StandardText
        );
        assert_eq!(
            FileCategory::from_extension("toml"),
            FileCategory::StandardText
        );
        assert_eq!(
            FileCategory::from_extension("txt"),
            FileCategory::StandardText
        );
    }

    #[test]
    fn file_category_from_extension_unknown() {
        // Validates: Requirement 4.5 — unknown extension falls back to Unknown
        assert_eq!(
            FileCategory::from_extension("xyz123"),
            FileCategory::Unknown
        );
        assert_eq!(FileCategory::from_extension(""), FileCategory::Unknown);
    }

    #[test]
    fn node_type_expandable_classification() {
        // Validates: Requirement 3.1 — expandable node types
        assert!(NodeType::Directory.is_expandable());
        assert!(NodeType::RootCategory.is_expandable());
        assert!(NodeType::DatasetPartitioned.is_expandable());
        assert!(!NodeType::File.is_expandable());
        assert!(!NodeType::PdsMember.is_expandable());
        assert!(!NodeType::Placeholder.is_expandable());
    }

    #[test]
    fn tree_node_hidden_detection() {
        // Validates: Requirement 4.7 — hidden files detected by leading dot
        let hidden = TreeNode::new(NodeId(1), NodeId::ROOT, ".hidden", NodeType::File, 1);
        assert!(hidden.is_hidden);
        let visible = TreeNode::new(NodeId(2), NodeId::ROOT, "visible.txt", NodeType::File, 1);
        assert!(!visible.is_hidden);
    }

    #[test]
    fn tree_node_category_from_type() {
        // Validates: Requirement 4.5 — category derived from node type
        let dir = TreeNode::new(NodeId(1), NodeId::ROOT, "src", NodeType::Directory, 1);
        assert_eq!(dir.category, FileCategory::Directory);
        let link = TreeNode::new(NodeId(2), NodeId::ROOT, "link", NodeType::SymbolicLink, 1);
        assert_eq!(link.category, FileCategory::SymbolicLink);
        let ds = TreeNode::new(
            NodeId(3),
            NodeId::ROOT,
            "HLQ.DATA",
            NodeType::DatasetSequential,
            1,
        );
        assert_eq!(ds.category, FileCategory::FileForgeStructured);
    }

    #[test]
    fn tree_node_data_directory_not_hidden() {
        let d = TreeNodeData::directory("src");
        assert!(!d.is_hidden);
        assert_eq!(d.node_type, NodeType::Directory);
    }

    #[test]
    fn tree_node_data_file_extension_classified() {
        let f = TreeNodeData::file("main.rs");
        assert_eq!(f.category, FileCategory::StandardText);
    }
}
