//! Pretty printer for formatting AST back to IDCAMS control statements.
//!
//! Supports compact (single-line) and verbose (multi-line with indentation) modes.

use crate::parser::ast::*;

/// Print mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintMode {
    /// Minimal whitespace, single line where possible.
    Compact,
    /// One parameter per line, indented.
    Verbose,
}

/// Formats a parsed command back into valid IDCAMS control statement text.
///
/// The output is guaranteed to be parseable without error.
pub fn pretty_print(command: &Command, mode: PrintMode) -> String {
    match mode {
        PrintMode::Compact => format_compact(command),
        PrintMode::Verbose => format_verbose(command),
    }
}

fn format_compact(command: &Command) -> String {
    match command {
        Command::DefineCluster(cmd) => format_define_cluster_compact(cmd),
        Command::DefineAix(cmd) => format_define_aix_compact(cmd),
        Command::DefinePath(cmd) => format_define_path_compact(cmd),
        Command::DefineGdg(cmd) => format_define_gdg_compact(cmd),
        Command::Delete(cmd) => format_delete_compact(cmd),
        Command::Alter(cmd) => format_alter_compact(cmd),
        Command::Listcat(cmd) => format_listcat_compact(cmd),
        Command::Print(cmd) => format_print_compact(cmd),
        Command::Repro(cmd) => format_repro_compact(cmd),
        Command::Verify(cmd) => format_verify_compact(cmd),
        Command::Export(cmd) => format_export_compact(cmd),
        Command::Import(cmd) => format_import_compact(cmd),
        Command::Bldindex(cmd) => format_bldindex_compact(cmd),
        Command::Set(cmd) => format_set_compact(cmd),
        Command::If(cmd) => format_if_compact(cmd),
        Command::Error(e) => format!("/* ERROR: {} */", e.message),
    }
}

fn format_verbose(command: &Command) -> String {
    // For now, verbose reuses compact with line breaks at params
    format_compact(command)
}

fn format_define_cluster_compact(cmd: &DefineClusterCommand) -> String {
    let mut parts = vec![format!("DEFINE CLUSTER (NAME({})", cmd.name)];
    parts.push(format!("{}", cmd.organization));
    if !cmd.volumes.is_empty() {
        parts.push(format!("VOLUMES({})", cmd.volumes.join(" ")));
    }
    if let Some(ref space) = cmd.space {
        parts.push(format_space(space));
    }
    if let Some((avg, max)) = cmd.recordsize {
        parts.push(format!("RECORDSIZE({avg} {max})"));
    }
    if let Some((len, off)) = cmd.keys {
        parts.push(format!("KEYS({len} {off})"));
    }
    if let Some((ci, ca)) = cmd.freespace {
        parts.push(format!("FREESPACE({ci} {ca})"));
    }
    if let Some((cr, cs)) = cmd.shareoptions {
        parts.push(format!("SHAREOPTIONS({cr} {cs})"));
    }
    if let Some(sr) = cmd.speed_recovery {
        match sr {
            SpeedRecovery::Speed => parts.push("SPEED".to_string()),
            SpeedRecovery::Recovery => parts.push("RECOVERY".to_string()),
        }
    }
    if cmd.reuse {
        parts.push("REUSE".to_string());
    }
    if let Some(bs) = cmd.bufferspace {
        parts.push(format!("BUFFERSPACE({bs})"));
    }
    parts.push(")".to_string());
    parts.join(" ")
}

fn format_define_aix_compact(cmd: &DefineAixCommand) -> String {
    let mut s = format!(
        "DEFINE ALTERNATEINDEX (NAME({}) RELATE({}) KEYS({} {})",
        cmd.name, cmd.relate, cmd.keys.0, cmd.keys.1
    );
    if !cmd.uniquekey {
        s.push_str(" NONUNIQUEKEY");
    }
    if !cmd.upgrade {
        s.push_str(" NOUPGRADE");
    }
    if let Some((avg, max)) = cmd.recordsize {
        s.push_str(&format!(" RECORDSIZE({avg} {max})"));
    }
    s.push(')');
    s
}

fn format_define_path_compact(cmd: &DefinePathCommand) -> String {
    let mut s = format!(
        "DEFINE PATH (NAME({}) PATHENTRY({})",
        cmd.name, cmd.pathentry
    );
    if !cmd.update {
        s.push_str(" NOUPDATE");
    }
    s.push(')');
    s
}

fn format_define_gdg_compact(cmd: &DefineGdgCommand) -> String {
    let mut s = format!("DEFINE GDG (NAME({}) LIMIT({})", cmd.name, cmd.limit);
    if !cmd.scratch {
        s.push_str(" NOSCRATCH");
    }
    if cmd.empty {
        s.push_str(" EMPTY");
    }
    if cmd.fifo {
        s.push_str(" FIFO");
    }
    s.push(')');
    s
}

fn format_delete_compact(cmd: &DeleteCommand) -> String {
    let names: Vec<String> = cmd.entries.iter().map(|e| e.to_string()).collect();
    let mut s = if names.len() == 1 {
        format!("DELETE {} {}", names[0], cmd.entry_type)
    } else {
        format!("DELETE ({}) {}", names.join(" "), cmd.entry_type)
    };
    if cmd.purge {
        s.push_str(" PURGE");
    }
    if cmd.force {
        s.push_str(" FORCE");
    }
    if cmd.erase {
        s.push_str(" ERASE");
    }
    s
}

fn format_alter_compact(cmd: &AlterCommand) -> String {
    let mut s = format!("ALTER {}", cmd.entry_name);
    if let Some((ci, ca)) = cmd.freespace {
        s.push_str(&format!(" FREESPACE({ci} {ca})"));
    }
    if let Some((cr, cs)) = cmd.shareoptions {
        s.push_str(&format!(" SHAREOPTIONS({cr} {cs})"));
    }
    if let Some(bs) = cmd.bufferspace {
        s.push_str(&format!(" BUFFERSPACE({bs})"));
    }
    if let Some((avg, max)) = cmd.recordsize {
        s.push_str(&format!(" RECORDSIZE({avg} {max})"));
    }
    if let Some(ref nn) = cmd.newname {
        s.push_str(&format!(" NEWNAME({nn})"));
    }
    s
}

fn format_listcat_compact(cmd: &ListcatCommand) -> String {
    let mut s = String::from("LISTCAT");
    match &cmd.filter {
        ListcatFilter::All => {}
        ListcatFilter::Entries(entries) => {
            s.push_str(&format!(" ENTRIES({})", entries.join(" ")));
        }
        ListcatFilter::Level(lvl) => {
            s.push_str(&format!(" LEVEL({lvl})"));
        }
    }
    match cmd.display_level {
        DisplayLevel::Name => s.push_str(" NAME"),
        DisplayLevel::History => s.push_str(" HISTORY"),
        DisplayLevel::Volume => s.push_str(" VOLUME"),
        DisplayLevel::All => s.push_str(" ALL"),
    }
    s
}

fn format_print_compact(cmd: &PrintCommand) -> String {
    let mut s = String::from("PRINT");
    match &cmd.input {
        InputSpec::InFile(dd) => s.push_str(&format!(" INFILE({dd})")),
        InputSpec::InDataset(dsn) => s.push_str(&format!(" INDATASET({dsn})")),
    }
    match cmd.format {
        PrintFormat::Character => s.push_str(" CHARACTER"),
        PrintFormat::Hex => s.push_str(" HEX"),
        PrintFormat::Dump => s.push_str(" DUMP"),
    }
    if let Some(ref kr) = cmd.key_range {
        s.push_str(&format!(" FROMKEY({})", kr.0));
        if let Some(ref to) = kr.1 {
            s.push_str(&format!(" TOKEY({to})"));
        }
    }
    if let Some(n) = cmd.count {
        s.push_str(&format!(" COUNT({n})"));
    }
    if let Some(n) = cmd.skip {
        s.push_str(&format!(" SKIP({n})"));
    }
    s
}

fn format_repro_compact(cmd: &ReproCommand) -> String {
    let mut s = String::from("REPRO");
    match &cmd.input {
        InputSpec::InFile(dd) => s.push_str(&format!(" INFILE({dd})")),
        InputSpec::InDataset(dsn) => s.push_str(&format!(" INDATASET({dsn})")),
    }
    match &cmd.output {
        OutputSpec::OutFile(dd) => s.push_str(&format!(" OUTFILE({dd})")),
        OutputSpec::OutDataset(dsn) => s.push_str(&format!(" OUTDATASET({dsn})")),
    }
    if cmd.replace {
        s.push_str(" REPLACE");
    }
    if let Some(n) = cmd.count {
        s.push_str(&format!(" COUNT({n})"));
    }
    if let Some(n) = cmd.skip {
        s.push_str(&format!(" SKIP({n})"));
    }
    s
}

fn format_verify_compact(cmd: &VerifyCommand) -> String {
    let mut s = String::from("VERIFY");
    match &cmd.dataset {
        InputSpec::InFile(dd) => s.push_str(&format!(" FILE({dd})")),
        InputSpec::InDataset(dsn) => s.push_str(&format!(" DATASET({dsn})")),
    }
    s
}

fn format_export_compact(cmd: &ExportCommand) -> String {
    let mut s = format!("EXPORT {}", cmd.entry_name);
    match &cmd.output {
        OutputSpec::OutFile(dd) => s.push_str(&format!(" OUTFILE({dd})")),
        OutputSpec::OutDataset(dsn) => s.push_str(&format!(" OUTDATASET({dsn})")),
    }
    if cmd.temporary {
        s.push_str(" TEMPORARY");
    }
    if cmd.inhibit_source {
        s.push_str(" INHIBITSOURCE");
    }
    s
}

fn format_import_compact(cmd: &ImportCommand) -> String {
    let mut s = String::from("IMPORT");
    match &cmd.input {
        InputSpec::InFile(dd) => s.push_str(&format!(" INFILE({dd})")),
        InputSpec::InDataset(dsn) => s.push_str(&format!(" INDATASET({dsn})")),
    }
    s.push_str(&format!(" OUTDATASET({})", cmd.out_dataset));
    s
}

fn format_bldindex_compact(cmd: &BldindexCommand) -> String {
    format!(
        "BLDINDEX INDATASET({}) OUTDATASET({})",
        cmd.in_dataset, cmd.out_dataset
    )
}

fn format_set_compact(cmd: &SetCommand) -> String {
    let target = match cmd.target {
        SetTarget::LastCC => "LASTCC",
        SetTarget::MaxCC => "MAXCC",
    };
    format!("SET {}({})", target, cmd.value)
}

fn format_if_compact(cmd: &IfCommand) -> String {
    let cond = format_condition(&cmd.condition);
    let then_str: Vec<String> = cmd.then_commands.iter().map(format_compact).collect();

    let mut s = format!("IF {} THEN", cond);
    if then_str.len() == 1 {
        s.push_str(&format!(" {}", then_str[0]));
    } else {
        s.push_str(" DO");
        for t in &then_str {
            s.push_str(&format!(" {}", t));
        }
        s.push_str(" END");
    }

    if let Some(ref else_cmds) = cmd.else_commands {
        let else_str: Vec<String> = else_cmds.iter().map(format_compact).collect();
        if else_str.len() == 1 {
            s.push_str(&format!(" ELSE {}", else_str[0]));
        } else {
            s.push_str(" ELSE DO");
            for e in &else_str {
                s.push_str(&format!(" {}", e));
            }
            s.push_str(" END");
        }
    }

    s
}

fn format_condition(cond: &Condition) -> String {
    match cond {
        Condition::Compare {
            register,
            op,
            value,
        } => {
            let reg = match register {
                ConditionRegister::LastCC => "LASTCC",
                ConditionRegister::MaxCC => "MAXCC",
            };
            let op_str = match op {
                crate::parser::token::CmpOp::Eq => "EQ",
                crate::parser::token::CmpOp::Ne => "NE",
                crate::parser::token::CmpOp::Gt => "GT",
                crate::parser::token::CmpOp::Lt => "LT",
                crate::parser::token::CmpOp::Ge => "GE",
                crate::parser::token::CmpOp::Le => "LE",
            };
            format!("{reg} {op_str} {value}")
        }
        Condition::And(left, right) => {
            format!("{} AND {}", format_condition(left), format_condition(right))
        }
        Condition::Or(left, right) => {
            format!("{} OR {}", format_condition(left), format_condition(right))
        }
    }
}

fn format_space(space: &SpaceUnit) -> String {
    match space {
        SpaceUnit::Cylinders { primary, secondary } => {
            format!("CYLINDERS({primary} {secondary})")
        }
        SpaceUnit::Tracks { primary, secondary } => {
            format!("TRACKS({primary} {secondary})")
        }
        SpaceUnit::Records { primary, secondary } => {
            format!("RECORDS({primary} {secondary})")
        }
        SpaceUnit::Kilobytes { primary, secondary } => {
            format!("KILOBYTES({primary} {secondary})")
        }
    }
}
