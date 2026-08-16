//! AST node definitions for parsed IDCAMS commands.

use std::fmt;

use super::token::CmpOp;

/// Dataset name: 1-44 characters, dot-separated qualifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetName(String);

impl DatasetName {
    /// Creates a new dataset name, uppercasing the input.
    ///
    /// # Errors
    ///
    /// Returns `None` if the name is empty or exceeds 44 characters.
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let s: String = name.into().to_uppercase();
        if s.is_empty() || s.len() > 44 {
            None
        } else {
            Some(Self(s))
        }
    }

    /// Creates a dataset name without validation (for testing/internal use).
    pub fn unchecked(name: impl Into<String>) -> Self {
        Self(name.into().to_uppercase())
    }

    /// Returns the dataset name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DatasetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The top-level command enum — each variant represents a parsed IDCAMS command.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// DEFINE CLUSTER command.
    DefineCluster(DefineClusterCommand),
    /// DEFINE ALTERNATEINDEX command.
    DefineAix(DefineAixCommand),
    /// DEFINE PATH command.
    DefinePath(DefinePathCommand),
    /// DEFINE GDG command.
    DefineGdg(DefineGdgCommand),
    /// DELETE command.
    Delete(DeleteCommand),
    /// ALTER command.
    Alter(AlterCommand),
    /// LISTCAT command.
    Listcat(ListcatCommand),
    /// PRINT command.
    Print(PrintCommand),
    /// REPRO command.
    Repro(ReproCommand),
    /// VERIFY command.
    Verify(VerifyCommand),
    /// EXPORT command.
    Export(ExportCommand),
    /// IMPORT command.
    Import(ImportCommand),
    /// BLDINDEX command.
    Bldindex(BldindexCommand),
    /// SET command.
    Set(SetCommand),
    /// IF/THEN/ELSE command.
    If(IfCommand),
    /// Error recovery node — produced when parsing fails.
    Error(ParseErrorNode),
}

/// An error node in the AST, marking a parse failure.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrorNode {
    /// The error message code.
    pub code: String,
    /// Human-readable error description.
    pub message: String,
    /// Position in input where the error occurred.
    pub position: usize,
}

/// Space allocation specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceUnit {
    /// CYLINDERS(primary secondary).
    Cylinders { primary: u32, secondary: u32 },
    /// TRACKS(primary secondary).
    Tracks { primary: u32, secondary: u32 },
    /// RECORDS(primary secondary).
    Records { primary: u32, secondary: u32 },
    /// KILOBYTES(primary secondary).
    Kilobytes { primary: u32, secondary: u32 },
}

/// VSAM organization type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsamOrganization {
    /// KSDS — Key Sequenced.
    Indexed,
    /// ESDS — Entry Sequenced.
    NonIndexed,
    /// RRDS — Relative Record.
    Numbered,
    /// LDS — Linear.
    Linear,
}

impl fmt::Display for VsamOrganization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Indexed => write!(f, "INDEXED"),
            Self::NonIndexed => write!(f, "NONINDEXED"),
            Self::Numbered => write!(f, "NUMBERED"),
            Self::Linear => write!(f, "LINEAR"),
        }
    }
}

/// SPEED vs RECOVERY option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedRecovery {
    /// Skip preformat.
    Speed,
    /// Preformat the dataset.
    Recovery,
}

/// DEFINE CLUSTER command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefineClusterCommand {
    /// Cluster name.
    pub name: DatasetName,
    /// Organization type.
    pub organization: VsamOrganization,
    /// Volume serials.
    pub volumes: Vec<String>,
    /// Space allocation.
    pub space: Option<SpaceUnit>,
    /// Record size (average, maximum).
    pub recordsize: Option<(u32, u32)>,
    /// Key definition (length, offset) — length 1-255.
    pub keys: Option<(u16, u32)>,
    /// Free space (CI percent, CA percent) 0-100.
    pub freespace: Option<(u8, u8)>,
    /// Share options (cross-region, cross-system) 1-4.
    pub shareoptions: Option<(u8, u8)>,
    /// SPEED or RECOVERY option.
    pub speed_recovery: Option<SpeedRecovery>,
    /// REUSE flag.
    pub reuse: bool,
    /// Buffer space in bytes.
    pub bufferspace: Option<u32>,
    /// DATA component definition.
    pub data_component: Option<ComponentDef>,
    /// INDEX component definition.
    pub index_component: Option<ComponentDef>,
}

/// Component sub-definition (DATA or INDEX within DEFINE CLUSTER).
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDef {
    /// Component name.
    pub name: Option<DatasetName>,
    /// Volume serials.
    pub volumes: Vec<String>,
    /// Space allocation.
    pub space: Option<SpaceUnit>,
    /// Record size (average, maximum).
    pub recordsize: Option<(u32, u32)>,
    /// Key definition (length, offset).
    pub keys: Option<(u16, u32)>,
    /// Control interval size.
    pub controlintervalsize: Option<u32>,
    /// Free space (CI percent, CA percent).
    pub freespace: Option<(u8, u8)>,
}

/// DEFINE ALTERNATEINDEX command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefineAixCommand {
    /// AIX name.
    pub name: DatasetName,
    /// Base cluster name.
    pub relate: DatasetName,
    /// Key definition (length, offset).
    pub keys: (u16, u32),
    /// true = UNIQUEKEY (default), false = NONUNIQUEKEY.
    pub uniquekey: bool,
    /// true = UPGRADE (default), false = NOUPGRADE.
    pub upgrade: bool,
    /// Record size for AIX.
    pub recordsize: Option<(u32, u32)>,
}

/// DEFINE PATH command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefinePathCommand {
    /// Path name.
    pub name: DatasetName,
    /// AIX name (PATHENTRY).
    pub pathentry: DatasetName,
    /// true = UPDATE (default), false = NOUPDATE.
    pub update: bool,
}

/// DEFINE GDG command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefineGdgCommand {
    /// GDG base name.
    pub name: DatasetName,
    /// Maximum generations (1-255).
    pub limit: u8,
    /// true = SCRATCH (default), false = NOSCRATCH.
    pub scratch: bool,
    /// true = EMPTY, false = NOEMPTY (default).
    pub empty: bool,
    /// true = FIFO, false = LIFO (default).
    pub fifo: bool,
}

/// DELETE command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteCommand {
    /// Entry names to delete.
    pub entries: Vec<DatasetName>,
    /// Entry type filter.
    pub entry_type: DeleteEntryType,
    /// PURGE flag.
    pub purge: bool,
    /// FORCE flag.
    pub force: bool,
    /// ERASE flag.
    pub erase: bool,
    /// SCRATCH flag (for GDG).
    pub scratch: Option<bool>,
}

/// Entry type for DELETE command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeleteEntryType {
    /// VSAM cluster.
    Cluster,
    /// Alternate index.
    AlternateIndex,
    /// Path.
    Path,
    /// Generation Data Group.
    Gdg,
    /// Non-VSAM dataset.
    NonVsam,
    /// User catalog.
    UserCatalog,
}

impl fmt::Display for DeleteEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cluster => write!(f, "CLUSTER"),
            Self::AlternateIndex => write!(f, "ALTERNATEINDEX"),
            Self::Path => write!(f, "PATH"),
            Self::Gdg => write!(f, "GDG"),
            Self::NonVsam => write!(f, "NONVSAM"),
            Self::UserCatalog => write!(f, "USERCATALOG"),
        }
    }
}

/// ALTER command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct AlterCommand {
    /// Entry name to alter.
    pub entry_name: DatasetName,
    /// New free space settings.
    pub freespace: Option<(u8, u8)>,
    /// New share options.
    pub shareoptions: Option<(u8, u8)>,
    /// New buffer space.
    pub bufferspace: Option<u32>,
    /// New record size.
    pub recordsize: Option<(u32, u32)>,
    /// New keys.
    pub keys: Option<(u16, u32)>,
    /// Volumes to add.
    pub add_volumes: Vec<String>,
    /// Volumes to remove.
    pub remove_volumes: Vec<String>,
    /// New name.
    pub newname: Option<DatasetName>,
    /// Attributes to nullify/reset.
    pub nullify: Vec<String>,
}

/// LISTCAT command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ListcatCommand {
    /// Filter criteria.
    pub filter: ListcatFilter,
    /// Display detail level.
    pub display_level: DisplayLevel,
    /// Catalog specification.
    pub catalog: Option<DatasetName>,
    /// Entry type filter.
    pub entry_type_filter: EntryTypeFilter,
}

/// Filter for LISTCAT command.
#[derive(Debug, Clone, PartialEq)]
pub enum ListcatFilter {
    /// List all entries.
    All,
    /// List specific entries (may contain wildcards).
    Entries(Vec<String>),
    /// Level filter (high-level qualifier).
    Level(String),
}

/// Display detail level for LISTCAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayLevel {
    /// Names only.
    Name,
    /// Names + history.
    History,
    /// Names + volume info.
    Volume,
    /// Complete attribute display.
    All,
}

/// Entry type filter for LISTCAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryTypeFilter {
    /// All entry types.
    All,
    /// Clusters only.
    Cluster,
    /// Alternate indexes only.
    AlternateIndex,
    /// Paths only.
    Path,
    /// GDG bases only.
    Gdg,
    /// Non-VSAM only.
    NonVsam,
    /// User catalogs only.
    UserCatalog,
    /// Data components only.
    Data,
    /// Index components only.
    Index,
}

/// PRINT command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrintCommand {
    /// Input dataset specification.
    pub input: InputSpec,
    /// Output format.
    pub format: PrintFormat,
    /// Key range (FROMKEY, TOKEY).
    pub key_range: Option<(String, Option<String>)>,
    /// Address range (FROMADDRESS, TOADDRESS).
    pub address_range: Option<(u64, Option<u64>)>,
    /// Record range (FROMRECORD, TORECORD).
    pub record_range: Option<(u64, Option<u64>)>,
    /// Maximum records to print.
    pub count: Option<u64>,
    /// Records to skip.
    pub skip: Option<u64>,
}

/// Input dataset specification (INFILE or INDATASET).
#[derive(Debug, Clone, PartialEq)]
pub enum InputSpec {
    /// INFILE(ddname).
    InFile(String),
    /// INDATASET(dsn).
    InDataset(DatasetName),
}

/// Output format for PRINT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintFormat {
    /// Printable characters with non-printable as periods.
    Character,
    /// Hexadecimal representation.
    Hex,
    /// Combined character and hex display.
    Dump,
}

/// REPRO command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReproCommand {
    /// Input source.
    pub input: InputSpec,
    /// Output target.
    pub output: OutputSpec,
    /// Key range (FROMKEY, TOKEY).
    pub key_range: Option<(String, Option<String>)>,
    /// Address range (FROMADDRESS, TOADDRESS).
    pub address_range: Option<(u64, Option<u64>)>,
    /// Maximum records to copy.
    pub count: Option<u64>,
    /// Records to skip.
    pub skip: Option<u64>,
    /// Whether to replace existing records with matching keys.
    pub replace: bool,
}

/// Output specification (OUTFILE or OUTDATASET).
#[derive(Debug, Clone, PartialEq)]
pub enum OutputSpec {
    /// OUTFILE(ddname).
    OutFile(String),
    /// OUTDATASET(dsn).
    OutDataset(DatasetName),
}

/// VERIFY command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifyCommand {
    /// Dataset to verify.
    pub dataset: InputSpec,
}

/// EXPORT command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportCommand {
    /// Source dataset name.
    pub entry_name: DatasetName,
    /// Output destination.
    pub output: OutputSpec,
    /// false = PERMANENT (default), true = TEMPORARY.
    pub temporary: bool,
    /// false = NOINHIBITSOURCE (default), true = INHIBITSOURCE.
    pub inhibit_source: bool,
}

/// IMPORT command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportCommand {
    /// Input source.
    pub input: InputSpec,
    /// Target dataset name.
    pub out_dataset: DatasetName,
    /// Catalog specification.
    pub catalog: Option<DatasetName>,
    /// Object mappings for rename/override.
    pub objects: Vec<ObjectMapping>,
}

/// Object mapping for IMPORT.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectMapping {
    /// Original object name.
    pub old_name: DatasetName,
    /// New name (if renaming).
    pub new_name: Option<DatasetName>,
    /// Volume overrides.
    pub volumes: Vec<String>,
}

/// BLDINDEX command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct BldindexCommand {
    /// Base cluster to scan.
    pub in_dataset: DatasetName,
    /// AIX to populate.
    pub out_dataset: DatasetName,
    /// Catalog specification.
    pub catalog: Option<DatasetName>,
}

/// SET command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct SetCommand {
    /// Which register to set.
    pub target: SetTarget,
    /// Value to set (0-16).
    pub value: u8,
}

/// Target register for SET.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetTarget {
    /// Set LASTCC.
    LastCC,
    /// Set MAXCC.
    MaxCC,
}

/// IF/THEN/ELSE command representation.
#[derive(Debug, Clone, PartialEq)]
pub struct IfCommand {
    /// Condition to evaluate.
    pub condition: Condition,
    /// Commands to execute if condition is true.
    pub then_commands: Vec<Command>,
    /// Commands to execute if condition is false.
    pub else_commands: Option<Vec<Command>>,
}

/// Condition expression for IF statements.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Simple comparison: register op value.
    Compare {
        /// Which register (LASTCC or MAXCC).
        register: ConditionRegister,
        /// Comparison operator.
        op: CmpOp,
        /// Value to compare against.
        value: u8,
    },
    /// Logical AND of two conditions.
    And(Box<Condition>, Box<Condition>),
    /// Logical OR of two conditions.
    Or(Box<Condition>, Box<Condition>),
}

/// Register used in IF conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionRegister {
    /// LASTCC register.
    LastCC,
    /// MAXCC register.
    MaxCC,
}
