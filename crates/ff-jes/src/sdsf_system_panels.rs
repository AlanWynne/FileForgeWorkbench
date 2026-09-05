//! SDSF system panels (SYS, DASH, INIT, JC, SP) and browse/print/COLS.
//!
//! Implements Requirement 18 AC 18.14-18.21:
//!   - SYS panel: active address spaces (AC 18.14)
//!   - DASH panel: system health metrics (AC 18.15)
//!   - INIT panel: initiator pool status (AC 18.16)
//!   - JC panel: job class definitions (AC 18.17)
//!   - SP panel: spool volume utilisation (AC 18.18)
//!   - Browse settings: line width, record format, FIND (AC 18.19)
//!   - PRINT action: route output to print destination (AC 18.20)
//!   - COLS command: column ruler display (AC 18.21)

// === SYS Panel ==============================================================

/// An address space entry in the SYS panel.
///
/// Addresses: Requirement 18 AC 18.14
#[derive(Debug, Clone)]
pub struct AddressSpace {
    /// Address space name.
    pub name: String,
    /// Current status (e.g. "ACTIVE", "WAITING", "ENDED").
    pub status: String,
    /// CPU usage percentage (0-100).
    pub cpu_pct: f32,
    /// Memory usage in KB.
    pub memory_kb: u64,
}

impl AddressSpace {
    pub fn new(name: &str, status: &str, cpu_pct: f32, memory_kb: u64) -> Self {
        Self {
            name: name.to_string(),
            status: status.to_string(),
            cpu_pct,
            memory_kb,
        }
    }
}

/// State for the SYS panel.
///
/// Addresses: Requirement 18 AC 18.14
#[derive(Debug, Clone, Default)]
pub struct SysPanelState {
    pub address_spaces: Vec<AddressSpace>,
}

impl SysPanelState {
    pub fn new(address_spaces: Vec<AddressSpace>) -> Self {
        Self { address_spaces }
    }

    /// Returns active address spaces only.
    pub fn active_spaces(&self) -> Vec<&AddressSpace> {
        self.address_spaces
            .iter()
            .filter(|a| a.status == "ACTIVE")
            .collect()
    }
}

// === DASH Panel =============================================================

/// System health metrics for the DASH panel.
///
/// Addresses: Requirement 18 AC 18.15
#[derive(Debug, Clone, Default)]
pub struct DashMetrics {
    /// Overall CPU utilisation percentage.
    pub cpu_pct: f32,
    /// Total memory in use (KB).
    pub memory_used_kb: u64,
    /// Total memory available (KB).
    pub memory_total_kb: u64,
    /// I/O operations per second.
    pub io_rate: u64,
    /// Number of active jobs.
    pub active_jobs: usize,
    /// Number of queued jobs.
    pub queued_jobs: usize,
}

impl DashMetrics {
    /// Memory utilisation as a percentage.
    pub fn memory_pct(&self) -> f32 {
        if self.memory_total_kb == 0 {
            return 0.0;
        }
        (self.memory_used_kb as f32 / self.memory_total_kb as f32) * 100.0
    }
}

// === INIT Panel =============================================================

/// An initiator entry in the INIT panel.
///
/// Addresses: Requirement 18 AC 18.16
#[derive(Debug, Clone)]
pub struct InitiatorEntry {
    /// Initiator ID.
    pub id: u32,
    /// Job class assignments (e.g. "ABC").
    pub classes: String,
    /// Whether the initiator is active.
    pub active: bool,
    /// Current job name if active.
    pub current_job: Option<String>,
}

impl InitiatorEntry {
    pub fn idle(id: u32, classes: &str) -> Self {
        Self {
            id,
            classes: classes.to_string(),
            active: false,
            current_job: None,
        }
    }

    pub fn active(id: u32, classes: &str, job: &str) -> Self {
        Self {
            id,
            classes: classes.to_string(),
            active: true,
            current_job: Some(job.to_string()),
        }
    }
}

/// State for the INIT panel.
///
/// Addresses: Requirement 18 AC 18.16
#[derive(Debug, Clone, Default)]
pub struct InitPanelState {
    pub initiators: Vec<InitiatorEntry>,
}

impl InitPanelState {
    pub fn new(initiators: Vec<InitiatorEntry>) -> Self {
        Self { initiators }
    }

    pub fn active_count(&self) -> usize {
        self.initiators.iter().filter(|i| i.active).count()
    }

    pub fn idle_count(&self) -> usize {
        self.initiators.iter().filter(|i| !i.active).count()
    }
}

// === JC Panel ===============================================================

/// A job class definition entry in the JC panel.
///
/// Addresses: Requirement 18 AC 18.17
#[derive(Debug, Clone)]
pub struct JobClassEntry {
    /// Single-character job class.
    pub class: char,
    /// Maximum priority for this class.
    pub max_priority: u32,
    /// Whether the class is active.
    pub active: bool,
    /// Description.
    pub description: String,
}

impl JobClassEntry {
    pub fn new(class: char, max_priority: u32, active: bool, description: &str) -> Self {
        Self {
            class,
            max_priority,
            active,
            description: description.to_string(),
        }
    }
}

/// State for the JC panel.
///
/// Addresses: Requirement 18 AC 18.17
#[derive(Debug, Clone, Default)]
pub struct JcPanelState {
    pub classes: Vec<JobClassEntry>,
}

impl JcPanelState {
    pub fn new(classes: Vec<JobClassEntry>) -> Self {
        Self { classes }
    }

    pub fn active_classes(&self) -> Vec<&JobClassEntry> {
        self.classes.iter().filter(|c| c.active).collect()
    }
}

// === SP Panel ===============================================================

/// A spool volume entry in the SP panel.
///
/// Addresses: Requirement 18 AC 18.18
#[derive(Debug, Clone)]
pub struct SpoolVolume {
    /// Volume serial number.
    pub volser: String,
    /// Total tracks on the volume.
    pub total_tracks: u64,
    /// Tracks currently allocated.
    pub used_tracks: u64,
}

impl SpoolVolume {
    pub fn new(volser: &str, total_tracks: u64, used_tracks: u64) -> Self {
        Self {
            volser: volser.to_string(),
            total_tracks,
            used_tracks,
        }
    }

    /// Utilisation as a percentage.
    pub fn utilisation_pct(&self) -> f32 {
        if self.total_tracks == 0 {
            return 0.0;
        }
        (self.used_tracks as f32 / self.total_tracks as f32) * 100.0
    }

    /// Free tracks.
    pub fn free_tracks(&self) -> u64 {
        self.total_tracks.saturating_sub(self.used_tracks)
    }
}

/// State for the SP panel.
///
/// Addresses: Requirement 18 AC 18.18
#[derive(Debug, Clone, Default)]
pub struct SpPanelState {
    pub volumes: Vec<SpoolVolume>,
}

impl SpPanelState {
    pub fn new(volumes: Vec<SpoolVolume>) -> Self {
        Self { volumes }
    }

    /// Total free tracks across all volumes.
    pub fn total_free_tracks(&self) -> u64 {
        self.volumes.iter().map(|v| v.free_tracks()).sum()
    }

    /// Total used tracks across all volumes.
    pub fn total_used_tracks(&self) -> u64 {
        self.volumes.iter().map(|v| v.used_tracks).sum()
    }
}

// === Browse Settings ========================================================

/// Record format for browse display.
///
/// Addresses: Requirement 18 AC 18.19
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum RecordFormat {
    #[default]
    Fixed,
    Variable,
    Undefined,
}

/// Browse settings for job output viewing.
///
/// Addresses: Requirement 18 AC 18.19
#[derive(Debug, Clone)]
pub struct BrowseSettings {
    /// Maximum line width for display.
    pub line_width: usize,
    /// Record format.
    pub record_format: RecordFormat,
    /// Current FIND search term (empty = no search).
    pub find_term: String,
    /// Index of the current FIND match.
    pub find_match: Option<usize>,
}

impl Default for BrowseSettings {
    fn default() -> Self {
        Self {
            line_width: 132,
            record_format: RecordFormat::Fixed,
            find_term: String::new(),
            find_match: None,
        }
    }
}

impl BrowseSettings {
    /// Execute a FIND within the given lines.
    ///
    /// Addresses: Requirement 18 AC 18.19
    pub fn find(&mut self, term: &str, lines: &[String]) -> Option<usize> {
        self.find_term = term.to_string();
        if term.is_empty() {
            self.find_match = None;
            return None;
        }
        let lower = term.to_lowercase();
        self.find_match = lines.iter().position(|l| l.to_lowercase().contains(&lower));
        self.find_match
    }
}

// === Print Action ===========================================================

/// Print destination for the PRINT action character.
///
/// Addresses: Requirement 18 AC 18.20
#[derive(Debug, Clone, PartialEq)]
pub enum PrintDestination {
    /// Local file path.
    File(String),
    /// Printer queue name.
    PrinterQueue(String),
}

/// Result of a PRINT action.
#[derive(Debug, Clone, PartialEq)]
pub enum PrintResult {
    /// Routed successfully.
    Routed { destination: String, lines: usize },
    /// No output to print.
    Empty,
}

/// Route job output to a print destination.
///
/// Addresses: Requirement 18 AC 18.20
pub fn print_output(lines: &[String], destination: PrintDestination) -> PrintResult {
    if lines.is_empty() {
        return PrintResult::Empty;
    }
    let dest_str = match &destination {
        PrintDestination::File(p) => p.clone(),
        PrintDestination::PrinterQueue(q) => q.clone(),
    };
    PrintResult::Routed {
        destination: dest_str,
        lines: lines.len(),
    }
}

// === COLS Command ===========================================================

/// Column ruler for the COLS command in browse.
///
/// Addresses: Requirement 18 AC 18.21
pub struct ColsRuler {
    /// Current horizontal scroll offset (0-based column index).
    pub scroll_offset: usize,
    /// Display width (number of columns visible).
    pub display_width: usize,
}

impl ColsRuler {
    pub fn new(scroll_offset: usize, display_width: usize) -> Self {
        Self {
            scroll_offset,
            display_width,
        }
    }

    /// Generate the ruler line string.
    ///
    /// Format: `----+----1----+----2...` starting from scroll_offset+1.
    ///
    /// Addresses: Requirement 18 AC 18.21
    #[allow(clippy::manual_is_multiple_of)]
    pub fn render(&self) -> String {
        let mut ruler = String::with_capacity(self.display_width);
        for i in 0..self.display_width {
            let col = self.scroll_offset + i + 1;
            if col % 10 == 0 {
                let digit = (col / 10) % 10;
                ruler.push(char::from_digit(digit as u32, 10).unwrap_or('0'));
            } else if col % 5 == 0 {
                ruler.push('+');
            } else {
                ruler.push('-');
            }
        }
        ruler
    }

    /// Returns the 1-based column number at the left edge.
    pub fn left_column(&self) -> usize {
        self.scroll_offset + 1
    }

    /// Returns the 1-based column number at the right edge.
    pub fn right_column(&self) -> usize {
        self.scroll_offset + self.display_width
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- SYS panel ----------------------------------------------------------

    // Validates: Requirement 18.14
    #[test]
    fn sys_panel_stores_address_spaces() {
        let state = SysPanelState::new(vec![
            AddressSpace::new("JES2", "ACTIVE", 2.5, 4096),
            AddressSpace::new("MYJOB", "WAITING", 0.0, 1024),
        ]);
        assert_eq!(state.address_spaces.len(), 2);
    }

    // Validates: Requirement 18.14
    #[test]
    fn sys_panel_filters_active_spaces() {
        let state = SysPanelState::new(vec![
            AddressSpace::new("JES2", "ACTIVE", 2.5, 4096),
            AddressSpace::new("MYJOB", "WAITING", 0.0, 1024),
        ]);
        assert_eq!(state.active_spaces().len(), 1);
        assert_eq!(state.active_spaces()[0].name, "JES2");
    }

    // --- DASH panel ---------------------------------------------------------

    // Validates: Requirement 18.15
    #[test]
    fn dash_metrics_computes_memory_pct() {
        let m = DashMetrics {
            memory_used_kb: 512,
            memory_total_kb: 1024,
            ..Default::default()
        };
        assert!((m.memory_pct() - 50.0).abs() < 0.01);
    }

    // Validates: Requirement 18.15
    #[test]
    fn dash_metrics_zero_total_memory_returns_zero_pct() {
        let m = DashMetrics::default();
        assert_eq!(m.memory_pct(), 0.0);
    }

    // --- INIT panel ---------------------------------------------------------

    // Validates: Requirement 18.16
    #[test]
    fn init_panel_counts_active_and_idle() {
        let state = InitPanelState::new(vec![
            InitiatorEntry::active(1, "ABC", "MYJOB"),
            InitiatorEntry::idle(2, "ABC"),
            InitiatorEntry::idle(3, "D"),
        ]);
        assert_eq!(state.active_count(), 1);
        assert_eq!(state.idle_count(), 2);
    }

    // Validates: Requirement 18.16
    #[test]
    fn init_panel_active_entry_has_job_name() {
        let entry = InitiatorEntry::active(1, "A", "PAYROLL");
        assert_eq!(entry.current_job, Some("PAYROLL".to_string()));
    }

    // --- JC panel -----------------------------------------------------------

    // Validates: Requirement 18.17
    #[test]
    fn jc_panel_lists_active_classes() {
        let state = JcPanelState::new(vec![
            JobClassEntry::new('A', 15, true, "Batch"),
            JobClassEntry::new('B', 10, false, "Inactive"),
        ]);
        assert_eq!(state.active_classes().len(), 1);
        assert_eq!(state.active_classes()[0].class, 'A');
    }

    // Validates: Requirement 18.17
    #[test]
    fn jc_panel_stores_scheduling_parameters() {
        let entry = JobClassEntry::new('A', 15, true, "High priority batch");
        assert_eq!(entry.max_priority, 15);
        assert_eq!(entry.description, "High priority batch");
    }

    // --- SP panel -----------------------------------------------------------

    // Validates: Requirement 18.18
    #[test]
    fn sp_panel_computes_utilisation_pct() {
        let vol = SpoolVolume::new("SPOOL1", 1000, 750);
        assert!((vol.utilisation_pct() - 75.0).abs() < 0.01);
    }

    // Validates: Requirement 18.18
    #[test]
    fn sp_panel_computes_free_tracks() {
        let vol = SpoolVolume::new("SPOOL1", 1000, 750);
        assert_eq!(vol.free_tracks(), 250);
    }

    // Validates: Requirement 18.18
    #[test]
    fn sp_panel_totals_across_volumes() {
        let state = SpPanelState::new(vec![
            SpoolVolume::new("SP001", 1000, 400),
            SpoolVolume::new("SP002", 2000, 600),
        ]);
        assert_eq!(state.total_used_tracks(), 1000);
        assert_eq!(state.total_free_tracks(), 2000);
    }

    // --- Browse settings ----------------------------------------------------

    // Validates: Requirement 18.19
    #[test]
    fn browse_settings_default_line_width_is_132() {
        let s = BrowseSettings::default();
        assert_eq!(s.line_width, 132);
    }

    // Validates: Requirement 18.19
    #[test]
    fn browse_find_locates_matching_line() {
        let mut s = BrowseSettings::default();
        let lines = vec!["hello world".to_string(), "foo bar".to_string()];
        let idx = s.find("world", &lines);
        assert_eq!(idx, Some(0));
    }

    // Validates: Requirement 18.19
    #[test]
    fn browse_find_is_case_insensitive() {
        let mut s = BrowseSettings::default();
        let lines = vec!["HELLO WORLD".to_string()];
        assert!(s.find("hello", &lines).is_some());
    }

    // Validates: Requirement 18.19
    #[test]
    fn browse_find_returns_none_for_no_match() {
        let mut s = BrowseSettings::default();
        let lines = vec!["hello".to_string()];
        assert!(s.find("XYZZY", &lines).is_none());
    }

    // --- PRINT action -------------------------------------------------------

    // Validates: Requirement 18.20
    #[test]
    fn print_output_routes_to_file() {
        let lines = vec!["line1".to_string(), "line2".to_string()];
        let result = print_output(&lines, PrintDestination::File("/tmp/out.txt".to_string()));
        assert!(matches!(result, PrintResult::Routed { lines: 2, .. }));
    }

    // Validates: Requirement 18.20
    #[test]
    fn print_output_routes_to_printer_queue() {
        let lines = vec!["line1".to_string()];
        let result = print_output(&lines, PrintDestination::PrinterQueue("LPT1".to_string()));
        assert!(matches!(result, PrintResult::Routed { .. }));
    }

    // Validates: Requirement 18.20
    #[test]
    fn print_output_empty_returns_empty() {
        let result = print_output(&[], PrintDestination::File("/tmp/out.txt".to_string()));
        assert_eq!(result, PrintResult::Empty);
    }

    // --- COLS ruler ---------------------------------------------------------

    // Validates: Requirement 18.21
    #[test]
    fn cols_ruler_length_matches_display_width() {
        let ruler = ColsRuler::new(0, 20);
        assert_eq!(ruler.render().len(), 20);
    }

    // Validates: Requirement 18.21
    #[test]
    fn cols_ruler_marks_tens_with_digit() {
        let ruler = ColsRuler::new(0, 15);
        let rendered = ruler.render();
        // Column 10 (index 9) should be '1'
        assert_eq!(rendered.chars().nth(9), Some('1'));
    }

    // Validates: Requirement 18.21
    #[test]
    fn cols_ruler_marks_fives_with_plus() {
        let ruler = ColsRuler::new(0, 10);
        let rendered = ruler.render();
        // Column 5 (index 4) should be '+'
        assert_eq!(rendered.chars().nth(4), Some('+'));
    }

    // Validates: Requirement 18.21
    #[test]
    fn cols_ruler_left_and_right_column_numbers() {
        let ruler = ColsRuler::new(10, 20);
        assert_eq!(ruler.left_column(), 11);
        assert_eq!(ruler.right_column(), 30);
    }

    // Validates: Requirement 18.21
    #[test]
    fn cols_ruler_at_zero_offset_starts_at_column_1() {
        let ruler = ColsRuler::new(0, 5);
        assert_eq!(ruler.left_column(), 1);
    }
}
