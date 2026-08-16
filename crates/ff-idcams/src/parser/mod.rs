//! IDCAMS control statement parser.
//!
//! Provides lexing, AST definition, and recursive-descent parsing for all
//! IDCAMS commands. Error recovery produces Error AST nodes rather than aborting.

pub mod ast;
pub mod lexer;
pub mod token;

use ast::*;
use lexer::Lexer;
use token::{CmpOp, Token, Verb};

/// The IDCAMS parser. Stateless — all state is per-invocation.
pub struct IdcamsParser;

impl IdcamsParser {
    /// Parses IDCAMS control statement text into a sequence of commands.
    ///
    /// Error recovery: produces `Command::Error` nodes for unrecognized verbs
    /// or malformed parameters rather than aborting.
    pub fn parse(input: &str) -> Vec<Command> {
        let tokens = Lexer::tokenize(input);
        let mut parser = ParserState::new(tokens);
        parser.parse_commands()
    }
}

/// Internal parser state for recursive-descent parsing.
struct ParserState {
    tokens: Vec<Token>,
    pos: usize,
}

impl ParserState {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let tok = self.tokens.get(self.pos).unwrap_or(&Token::Eof);
        self.pos += 1;
        tok
    }

    fn at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn skip_semicolons(&mut self) {
        while matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }
    }

    fn expect_open_paren(&mut self) -> bool {
        if matches!(self.peek(), Token::OpenParen) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_close_paren(&mut self) -> bool {
        if matches!(self.peek(), Token::CloseParen) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn read_string(&mut self) -> Option<String> {
        match self.peek().clone() {
            Token::StringLit(s) => {
                self.advance();
                Some(s)
            }
            Token::Keyword(s) => {
                self.advance();
                Some(s)
            }
            Token::Number(n) => {
                self.advance();
                Some(n.to_string())
            }
            _ => None,
        }
    }

    fn read_number(&mut self) -> Option<i64> {
        if let Token::Number(n) = self.peek() {
            let n = *n;
            self.advance();
            Some(n)
        } else {
            None
        }
    }

    fn read_dsn(&mut self) -> Option<DatasetName> {
        let s = self.read_string()?;
        DatasetName::new(s)
    }

    fn read_paren_string(&mut self) -> Option<String> {
        if !self.expect_open_paren() {
            return None;
        }
        let s = self.read_string();
        self.expect_close_paren();
        s
    }

    fn read_paren_dsn(&mut self) -> Option<DatasetName> {
        if !self.expect_open_paren() {
            return None;
        }
        let dsn = self.read_dsn();
        self.expect_close_paren();
        dsn
    }

    fn read_paren_number(&mut self) -> Option<i64> {
        if !self.expect_open_paren() {
            return None;
        }
        let n = self.read_number();
        self.expect_close_paren();
        n
    }

    fn read_paren_two_numbers(&mut self) -> Option<(i64, i64)> {
        if !self.expect_open_paren() {
            return None;
        }
        let a = self.read_number()?;
        let b = self.read_number()?;
        self.expect_close_paren();
        Some((a, b))
    }

    fn read_paren_string_list(&mut self) -> Vec<String> {
        let mut items = Vec::new();
        if !self.expect_open_paren() {
            return items;
        }
        while !matches!(self.peek(), Token::CloseParen | Token::Eof) {
            if let Some(s) = self.read_string() {
                items.push(s);
            } else {
                break;
            }
        }
        self.expect_close_paren();
        items
    }

    /// Skips tokens until the next command verb or end of input.
    fn skip_to_next_command(&mut self) {
        while !self.at_end() {
            match self.peek() {
                Token::Verb(_) | Token::Semicolon => break,
                _ => {
                    self.advance();
                }
            }
        }
        self.skip_semicolons();
    }

    fn parse_commands(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        self.skip_semicolons();

        while !self.at_end() {
            let cmd = self.parse_command();
            commands.push(cmd);
            self.skip_semicolons();
        }

        commands
    }

    fn parse_command(&mut self) -> Command {
        match self.peek().clone() {
            Token::Verb(verb) => {
                self.advance();
                match verb {
                    Verb::Define => self.parse_define(),
                    Verb::Delete => self.parse_delete(),
                    Verb::Alter => self.parse_alter(),
                    Verb::Listcat => self.parse_listcat(),
                    Verb::Print => self.parse_print(),
                    Verb::Repro => self.parse_repro(),
                    Verb::Verify => self.parse_verify(),
                    Verb::Export => self.parse_export(),
                    Verb::Import => self.parse_import(),
                    Verb::Bldindex => self.parse_bldindex(),
                    Verb::Set => self.parse_set(),
                    Verb::If => self.parse_if(),
                }
            }
            Token::Keyword(ref kw) => {
                let kw = kw.clone();
                let pos = self.pos;
                self.skip_to_next_command();
                Command::Error(ParseErrorNode {
                    code: "IDC0001E".to_string(),
                    message: format!("unrecognized command verb: {kw}"),
                    position: pos,
                })
            }
            _ => {
                let pos = self.pos;
                self.skip_to_next_command();
                Command::Error(ParseErrorNode {
                    code: "IDC0001E".to_string(),
                    message: "expected command verb".to_string(),
                    position: pos,
                })
            }
        }
    }

    fn parse_define(&mut self) -> Command {
        // Expect a sub-type keyword: CLUSTER, ALTERNATEINDEX, PATH, GDG
        match self.peek().clone() {
            Token::Keyword(ref kw) => {
                let sub = kw.clone();
                self.advance();
                match sub.as_str() {
                    "CLUSTER" | "CL" => self.parse_define_cluster(),
                    "ALTERNATEINDEX" | "AIX" => self.parse_define_aix(),
                    "PATH" => self.parse_define_path(),
                    "GDG" | "GENERATIONDATAGROUP" => self.parse_define_gdg(),
                    _ => Command::Error(ParseErrorNode {
                        code: "IDC0001E".to_string(),
                        message: format!("unrecognized DEFINE sub-type: {sub}"),
                        position: self.pos,
                    }),
                }
            }
            Token::OpenParen => {
                // DEFINE (NAME(x) ...) — shorthand for DEFINE CLUSTER
                self.parse_define_cluster()
            }
            _ => {
                let pos = self.pos;
                self.skip_to_next_command();
                Command::Error(ParseErrorNode {
                    code: "IDC0001E".to_string(),
                    message: "expected DEFINE sub-type".to_string(),
                    position: pos,
                })
            }
        }
    }

    fn parse_define_cluster(&mut self) -> Command {
        let mut name = None;
        let mut organization = VsamOrganization::Indexed;
        let mut volumes = Vec::new();
        let mut space = None;
        let mut recordsize = None;
        let mut keys = None;
        let mut freespace = None;
        let mut shareoptions = None;
        let mut speed_recovery = None;
        let mut reuse = false;
        let mut bufferspace = None;
        let mut data_component = None;
        let mut index_component = None;

        // Check for opening paren wrapping all parameters
        let has_outer_paren = self.expect_open_paren();

        loop {
            match self.peek().clone() {
                Token::CloseParen | Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "NAME" => name = self.read_paren_dsn(),
                        "INDEXED" => organization = VsamOrganization::Indexed,
                        "NONINDEXED" => organization = VsamOrganization::NonIndexed,
                        "NUMBERED" => organization = VsamOrganization::Numbered,
                        "LINEAR" => organization = VsamOrganization::Linear,
                        "VOLUMES" | "VOL" => volumes = self.read_paren_string_list(),
                        "CYLINDERS" | "CYL" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                space = Some(SpaceUnit::Cylinders {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "TRACKS" | "TRK" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                space = Some(SpaceUnit::Tracks {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "RECORDS" | "REC" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                space = Some(SpaceUnit::Records {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "KILOBYTES" | "KB" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                space = Some(SpaceUnit::Kilobytes {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "RECORDSIZE" | "RECSZ" => {
                            if let Some((a, m)) = self.read_paren_two_numbers() {
                                recordsize = Some((a as u32, m as u32));
                            }
                        }
                        "KEYS" => {
                            if let Some((l, o)) = self.read_paren_two_numbers() {
                                keys = Some((l as u16, o as u32));
                            }
                        }
                        "FREESPACE" | "FSPC" => {
                            if let Some((ci, ca)) = self.read_paren_two_numbers() {
                                freespace = Some((ci as u8, ca as u8));
                            }
                        }
                        "SHAREOPTIONS" | "SHR" => {
                            if let Some((cr, cs)) = self.read_paren_two_numbers() {
                                shareoptions = Some((cr as u8, cs as u8));
                            }
                        }
                        "SPEED" => speed_recovery = Some(SpeedRecovery::Speed),
                        "RECOVERY" => speed_recovery = Some(SpeedRecovery::Recovery),
                        "REUSE" => reuse = true,
                        "NOREUSE" => reuse = false,
                        "BUFFERSPACE" | "BUFSPC" => {
                            bufferspace = self.read_paren_number().map(|n| n as u32);
                        }
                        "CONTROLINTERVALSIZE" | "CISZ" => {
                            // Ignore at cluster level — only valid on components
                            let _ = self.read_paren_number();
                        }
                        "DATA" => data_component = Some(self.parse_component_def()),
                        "INDEX" => index_component = Some(self.parse_component_def()),
                        _ => {
                            // Skip unknown parameter
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        if has_outer_paren {
            self.expect_close_paren();
        }

        let dsn = name.unwrap_or_else(|| DatasetName::unchecked("UNNAMED"));
        Command::DefineCluster(DefineClusterCommand {
            name: dsn,
            organization,
            volumes,
            space,
            recordsize,
            keys,
            freespace,
            shareoptions,
            speed_recovery,
            reuse,
            bufferspace,
            data_component,
            index_component,
        })
    }

    fn parse_component_def(&mut self) -> ComponentDef {
        let mut comp = ComponentDef {
            name: None,
            volumes: Vec::new(),
            space: None,
            recordsize: None,
            keys: None,
            controlintervalsize: None,
            freespace: None,
        };

        if !self.expect_open_paren() {
            return comp;
        }

        loop {
            match self.peek().clone() {
                Token::CloseParen | Token::Eof => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "NAME" => comp.name = self.read_paren_dsn(),
                        "VOLUMES" | "VOL" => comp.volumes = self.read_paren_string_list(),
                        "CYLINDERS" | "CYL" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                comp.space = Some(SpaceUnit::Cylinders {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "TRACKS" | "TRK" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                comp.space = Some(SpaceUnit::Tracks {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "RECORDS" | "REC" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                comp.space = Some(SpaceUnit::Records {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "KILOBYTES" | "KB" => {
                            if let Some((p, s)) = self.read_paren_two_numbers() {
                                comp.space = Some(SpaceUnit::Kilobytes {
                                    primary: p as u32,
                                    secondary: s as u32,
                                });
                            }
                        }
                        "RECORDSIZE" | "RECSZ" => {
                            if let Some((a, m)) = self.read_paren_two_numbers() {
                                comp.recordsize = Some((a as u32, m as u32));
                            }
                        }
                        "KEYS" => {
                            if let Some((l, o)) = self.read_paren_two_numbers() {
                                comp.keys = Some((l as u16, o as u32));
                            }
                        }
                        "CONTROLINTERVALSIZE" | "CISZ" => {
                            comp.controlintervalsize = self.read_paren_number().map(|n| n as u32);
                        }
                        "FREESPACE" | "FSPC" => {
                            if let Some((ci, ca)) = self.read_paren_two_numbers() {
                                comp.freespace = Some((ci as u8, ca as u8));
                            }
                        }
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        self.expect_close_paren();
        comp
    }

    fn skip_paren_group(&mut self) {
        if !self.expect_open_paren() {
            return;
        }
        let mut depth = 1;
        while depth > 0 && !self.at_end() {
            match self.advance() {
                Token::OpenParen => depth += 1,
                Token::CloseParen => depth -= 1,
                _ => {}
            }
        }
    }

    fn parse_define_aix(&mut self) -> Command {
        let mut name = None;
        let mut relate = None;
        let mut keys = None;
        let mut uniquekey = true;
        let mut upgrade = true;
        let mut recordsize = None;

        let has_outer_paren = self.expect_open_paren();

        loop {
            match self.peek().clone() {
                Token::CloseParen | Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "NAME" => name = self.read_paren_dsn(),
                        "RELATE" => relate = self.read_paren_dsn(),
                        "KEYS" => {
                            if let Some((l, o)) = self.read_paren_two_numbers() {
                                keys = Some((l as u16, o as u32));
                            }
                        }
                        "UNIQUEKEY" | "UNIQK" => uniquekey = true,
                        "NONUNIQUEKEY" | "NUNIQK" => uniquekey = false,
                        "UPGRADE" => upgrade = true,
                        "NOUPGRADE" => upgrade = false,
                        "RECORDSIZE" | "RECSZ" => {
                            if let Some((a, m)) = self.read_paren_two_numbers() {
                                recordsize = Some((a as u32, m as u32));
                            }
                        }
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        if has_outer_paren {
            self.expect_close_paren();
        }

        let dsn = name.unwrap_or_else(|| DatasetName::unchecked("UNNAMED"));
        let rel = relate.unwrap_or_else(|| DatasetName::unchecked("UNNAMED"));
        let k = keys.unwrap_or((8, 0));

        Command::DefineAix(DefineAixCommand {
            name: dsn,
            relate: rel,
            keys: k,
            uniquekey,
            upgrade,
            recordsize,
        })
    }

    fn parse_define_path(&mut self) -> Command {
        let mut name = None;
        let mut pathentry = None;
        let mut update = true;

        let has_outer_paren = self.expect_open_paren();

        loop {
            match self.peek().clone() {
                Token::CloseParen | Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "NAME" => name = self.read_paren_dsn(),
                        "PATHENTRY" => pathentry = self.read_paren_dsn(),
                        "UPDATE" => update = true,
                        "NOUPDATE" => update = false,
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        if has_outer_paren {
            self.expect_close_paren();
        }

        Command::DefinePath(DefinePathCommand {
            name: name.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            pathentry: pathentry.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            update,
        })
    }

    fn parse_define_gdg(&mut self) -> Command {
        let mut name = None;
        let mut limit = None;
        let mut scratch = true;
        let mut empty = false;
        let mut fifo = false;

        let has_outer_paren = self.expect_open_paren();

        loop {
            match self.peek().clone() {
                Token::CloseParen | Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "NAME" => name = self.read_paren_dsn(),
                        "LIMIT" => limit = self.read_paren_number().map(|n| n as u8),
                        "SCRATCH" => scratch = true,
                        "NOSCRATCH" => scratch = false,
                        "EMPTY" => empty = true,
                        "NOEMPTY" => empty = false,
                        "FIFO" => fifo = true,
                        "LIFO" => fifo = false,
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        if has_outer_paren {
            self.expect_close_paren();
        }

        Command::DefineGdg(DefineGdgCommand {
            name: name.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            limit: limit.unwrap_or(0),
            scratch,
            empty,
            fifo,
        })
    }

    fn parse_delete(&mut self) -> Command {
        let mut entries = Vec::new();
        let mut entry_type = DeleteEntryType::Cluster;
        let mut purge = false;
        let mut force = false;
        let mut erase = false;
        let mut scratch = None;

        // Check for name list in parens or single name
        if matches!(self.peek(), Token::OpenParen) {
            self.advance();
            while !matches!(self.peek(), Token::CloseParen | Token::Eof) {
                if let Some(dsn) = self.read_dsn() {
                    entries.push(dsn);
                } else {
                    break;
                }
            }
            self.expect_close_paren();
        } else if let Some(dsn) = self.read_dsn() {
            entries.push(dsn);
        }

        // Parse options
        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "CLUSTER" | "CL" => entry_type = DeleteEntryType::Cluster,
                        "ALTERNATEINDEX" | "AIX" => entry_type = DeleteEntryType::AlternateIndex,
                        "PATH" => entry_type = DeleteEntryType::Path,
                        "GDG" | "GENERATIONDATAGROUP" => entry_type = DeleteEntryType::Gdg,
                        "NONVSAM" | "NVSAM" => entry_type = DeleteEntryType::NonVsam,
                        "USERCATALOG" | "UCAT" => entry_type = DeleteEntryType::UserCatalog,
                        "PURGE" => purge = true,
                        "NOPURGE" => purge = false,
                        "FORCE" => force = true,
                        "NOFORCE" => force = false,
                        "ERASE" => erase = true,
                        "NOERASE" => erase = false,
                        "SCRATCH" => scratch = Some(true),
                        "NOSCRATCH" => scratch = Some(false),
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Delete(DeleteCommand {
            entries,
            entry_type,
            purge,
            force,
            erase,
            scratch,
        })
    }

    fn parse_alter(&mut self) -> Command {
        let entry_name = self
            .read_dsn()
            .unwrap_or_else(|| DatasetName::unchecked("UNNAMED"));

        let mut cmd = AlterCommand {
            entry_name,
            freespace: None,
            shareoptions: None,
            bufferspace: None,
            recordsize: None,
            keys: None,
            add_volumes: Vec::new(),
            remove_volumes: Vec::new(),
            newname: None,
            nullify: Vec::new(),
        };

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "FREESPACE" | "FSPC" => {
                            cmd.freespace = self
                                .read_paren_two_numbers()
                                .map(|(ci, ca)| (ci as u8, ca as u8));
                        }
                        "SHAREOPTIONS" | "SHR" => {
                            cmd.shareoptions = self
                                .read_paren_two_numbers()
                                .map(|(cr, cs)| (cr as u8, cs as u8));
                        }
                        "BUFFERSPACE" | "BUFSPC" => {
                            cmd.bufferspace = self.read_paren_number().map(|n| n as u32);
                        }
                        "RECORDSIZE" | "RECSZ" => {
                            cmd.recordsize = self
                                .read_paren_two_numbers()
                                .map(|(a, m)| (a as u32, m as u32));
                        }
                        "KEYS" => {
                            cmd.keys = self
                                .read_paren_two_numbers()
                                .map(|(l, o)| (l as u16, o as u32));
                        }
                        "ADDVOLUMES" | "ADDVOL" => {
                            cmd.add_volumes = self.read_paren_string_list();
                        }
                        "REMOVEVOLUMES" | "RMVOL" => {
                            cmd.remove_volumes = self.read_paren_string_list();
                        }
                        "NEWNAME" => {
                            cmd.newname = self.read_paren_dsn();
                        }
                        "NULLIFY" => {
                            cmd.nullify = self.read_paren_string_list();
                        }
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Alter(cmd)
    }

    fn parse_listcat(&mut self) -> Command {
        let mut filter = ListcatFilter::All;
        let mut display_level = DisplayLevel::Name;
        let mut catalog = None;
        let mut entry_type_filter = EntryTypeFilter::All;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "ENTRIES" | "ENT" => {
                            let items = self.read_paren_string_list();
                            filter = ListcatFilter::Entries(items);
                        }
                        "LEVEL" | "LVL" => {
                            if let Some(s) = self.read_paren_string() {
                                filter = ListcatFilter::Level(s);
                            }
                        }
                        "NAME" => display_level = DisplayLevel::Name,
                        "HISTORY" | "HIST" => display_level = DisplayLevel::History,
                        "VOLUME" | "VOL" => display_level = DisplayLevel::Volume,
                        "ALL" => {
                            // Check context: is it display level or entry type?
                            // If it immediately follows ENTRIES/LEVEL keyword, it's type filter
                            // Otherwise treat as display level
                            display_level = DisplayLevel::All;
                        }
                        "CATALOG" | "CAT" => catalog = self.read_paren_dsn(),
                        "CLUSTER" | "CL" => entry_type_filter = EntryTypeFilter::Cluster,
                        "ALTERNATEINDEX" | "AIX" => {
                            entry_type_filter = EntryTypeFilter::AlternateIndex;
                        }
                        "PATH" => entry_type_filter = EntryTypeFilter::Path,
                        "GDG" => entry_type_filter = EntryTypeFilter::Gdg,
                        "NONVSAM" | "NVSAM" => entry_type_filter = EntryTypeFilter::NonVsam,
                        "USERCATALOG" | "UCAT" => {
                            entry_type_filter = EntryTypeFilter::UserCatalog;
                        }
                        "DATA" => entry_type_filter = EntryTypeFilter::Data,
                        "INDEX" => entry_type_filter = EntryTypeFilter::Index,
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Listcat(ListcatCommand {
            filter,
            display_level,
            catalog,
            entry_type_filter,
        })
    }

    fn parse_print(&mut self) -> Command {
        let mut input = None;
        let mut format = PrintFormat::Dump;
        let mut key_range = None;
        let mut address_range = None;
        let mut record_range = None;
        let mut count = None;
        let mut skip = None;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "INFILE" | "IFI" => {
                            input = self.read_paren_string().map(InputSpec::InFile);
                        }
                        "INDATASET" | "IDS" => {
                            input = self.read_paren_dsn().map(InputSpec::InDataset);
                        }
                        "CHARACTER" | "CHAR" => format = PrintFormat::Character,
                        "HEX" => format = PrintFormat::Hex,
                        "DUMP" => format = PrintFormat::Dump,
                        "FROMKEY" => {
                            let from = self.read_paren_string();
                            key_range = from.map(|f| (f, None));
                        }
                        "TOKEY" => {
                            if let Some(ref mut kr) = key_range {
                                kr.1 = self.read_paren_string();
                            }
                        }
                        "FROMADDRESS" => {
                            let from = self.read_paren_number().map(|n| n as u64);
                            address_range = from.map(|f| (f, None));
                        }
                        "TOADDRESS" => {
                            if let Some(ref mut ar) = address_range {
                                ar.1 = self.read_paren_number().map(|n| n as u64);
                            }
                        }
                        "FROMRECORD" | "FROMNUMBER" => {
                            let from = self.read_paren_number().map(|n| n as u64);
                            record_range = from.map(|f| (f, None));
                        }
                        "TORECORD" | "TONUMBER" => {
                            if let Some(ref mut rr) = record_range {
                                rr.1 = self.read_paren_number().map(|n| n as u64);
                            }
                        }
                        "COUNT" => count = self.read_paren_number().map(|n| n as u64),
                        "SKIP" => skip = self.read_paren_number().map(|n| n as u64),
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Print(PrintCommand {
            input: input.unwrap_or(InputSpec::InFile("SYSUT1".to_string())),
            format,
            key_range,
            address_range,
            record_range,
            count,
            skip,
        })
    }

    fn parse_repro(&mut self) -> Command {
        let mut input = None;
        let mut output = None;
        let mut key_range = None;
        let mut address_range = None;
        let mut count = None;
        let mut skip = None;
        let mut replace = false;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "INFILE" | "IFI" => {
                            input = self.read_paren_string().map(InputSpec::InFile);
                        }
                        "INDATASET" | "IDS" => {
                            input = self.read_paren_dsn().map(InputSpec::InDataset);
                        }
                        "OUTFILE" | "OFI" => {
                            output = self.read_paren_string().map(OutputSpec::OutFile);
                        }
                        "OUTDATASET" | "ODS" => {
                            output = self.read_paren_dsn().map(OutputSpec::OutDataset);
                        }
                        "FROMKEY" => {
                            let from = self.read_paren_string();
                            key_range = from.map(|f| (f, None));
                        }
                        "TOKEY" => {
                            if let Some(ref mut kr) = key_range {
                                kr.1 = self.read_paren_string();
                            }
                        }
                        "FROMADDRESS" => {
                            let from = self.read_paren_number().map(|n| n as u64);
                            address_range = from.map(|f| (f, None));
                        }
                        "TOADDRESS" => {
                            if let Some(ref mut ar) = address_range {
                                ar.1 = self.read_paren_number().map(|n| n as u64);
                            }
                        }
                        "COUNT" => count = self.read_paren_number().map(|n| n as u64),
                        "SKIP" => skip = self.read_paren_number().map(|n| n as u64),
                        "REPLACE" => replace = true,
                        "NOREPLACE" => replace = false,
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Repro(ReproCommand {
            input: input.unwrap_or(InputSpec::InFile("SYSUT1".to_string())),
            output: output.unwrap_or(OutputSpec::OutFile("SYSUT2".to_string())),
            key_range,
            address_range,
            count,
            skip,
            replace,
        })
    }

    fn parse_verify(&mut self) -> Command {
        let mut dataset = None;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "FILE" | "FI" => {
                            dataset = self.read_paren_string().map(InputSpec::InFile);
                        }
                        "DATASET" | "DS" => {
                            dataset = self.read_paren_dsn().map(InputSpec::InDataset);
                        }
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Verify(VerifyCommand {
            dataset: dataset.unwrap_or(InputSpec::InFile("SYSUT1".to_string())),
        })
    }

    fn parse_export(&mut self) -> Command {
        let mut entry_name = None;
        let mut output = None;
        let mut temporary = false;
        let mut inhibit_source = false;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "ENTRY" => entry_name = self.read_paren_dsn(),
                        "OUTFILE" | "OFI" => {
                            output = self.read_paren_string().map(OutputSpec::OutFile);
                        }
                        "OUTDATASET" | "ODS" => {
                            output = self.read_paren_dsn().map(OutputSpec::OutDataset);
                        }
                        "TEMPORARY" | "TEMP" => temporary = true,
                        "PERMANENT" | "PERM" => temporary = false,
                        "INHIBITSOURCE" | "IHBS" => inhibit_source = true,
                        "NOINHIBITSOURCE" | "NIHBS" => inhibit_source = false,
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                Token::StringLit(_) if entry_name.is_none() => {
                    entry_name = self.read_dsn();
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Export(ExportCommand {
            entry_name: entry_name.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            output: output.unwrap_or(OutputSpec::OutFile("SYSUT2".to_string())),
            temporary,
            inhibit_source,
        })
    }

    fn parse_import(&mut self) -> Command {
        let mut input = None;
        let mut out_dataset = None;
        let mut catalog = None;
        let mut objects = Vec::new();

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "INFILE" | "IFI" => {
                            input = self.read_paren_string().map(InputSpec::InFile);
                        }
                        "INDATASET" | "IDS" => {
                            input = self.read_paren_dsn().map(InputSpec::InDataset);
                        }
                        "OUTDATASET" | "ODS" => {
                            out_dataset = self.read_paren_dsn();
                        }
                        "CATALOG" | "CAT" => {
                            catalog = self.read_paren_dsn();
                        }
                        "OBJECTS" => {
                            objects = self.parse_object_mappings();
                        }
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Import(ImportCommand {
            input: input.unwrap_or(InputSpec::InFile("SYSUT1".to_string())),
            out_dataset: out_dataset.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            catalog,
            objects,
        })
    }

    fn parse_object_mappings(&mut self) -> Vec<ObjectMapping> {
        let mut mappings = Vec::new();
        if !self.expect_open_paren() {
            return mappings;
        }

        while !matches!(self.peek(), Token::CloseParen | Token::Eof) {
            if self.expect_open_paren() {
                let old_name = self
                    .read_dsn()
                    .unwrap_or_else(|| DatasetName::unchecked("UNNAMED"));
                let mut new_name = None;
                let mut volumes = Vec::new();

                while !matches!(self.peek(), Token::CloseParen | Token::Eof) {
                    if let Token::Keyword(ref kw) = self.peek().clone() {
                        let kw = kw.clone();
                        self.advance();
                        match kw.as_str() {
                            "NEWNAME" => new_name = self.read_paren_dsn(),
                            "VOLUMES" | "VOL" => volumes = self.read_paren_string_list(),
                            _ => {}
                        }
                    } else {
                        self.advance();
                    }
                }
                self.expect_close_paren();
                mappings.push(ObjectMapping {
                    old_name,
                    new_name,
                    volumes,
                });
            } else {
                break;
            }
        }
        self.expect_close_paren();
        mappings
    }

    fn parse_bldindex(&mut self) -> Command {
        let mut in_dataset = None;
        let mut out_dataset = None;
        let mut catalog = None;

        loop {
            match self.peek().clone() {
                Token::Eof | Token::Semicolon | Token::Verb(_) => break,
                Token::Keyword(ref kw) => {
                    let kw = kw.clone();
                    self.advance();
                    match kw.as_str() {
                        "INDATASET" | "IDS" => in_dataset = self.read_paren_dsn(),
                        "OUTDATASET" | "ODS" => out_dataset = self.read_paren_dsn(),
                        "CATALOG" | "CAT" => catalog = self.read_paren_dsn(),
                        _ => {
                            if matches!(self.peek(), Token::OpenParen) {
                                self.skip_paren_group();
                            }
                        }
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Command::Bldindex(BldindexCommand {
            in_dataset: in_dataset.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            out_dataset: out_dataset.unwrap_or_else(|| DatasetName::unchecked("UNNAMED")),
            catalog,
        })
    }

    fn parse_set(&mut self) -> Command {
        // SET MAXCC(n) or SET LASTCC(n)
        match self.peek().clone() {
            Token::Keyword(ref kw) => {
                let kw = kw.clone();
                self.advance();
                let target = match kw.as_str() {
                    "MAXCC" => SetTarget::MaxCC,
                    "LASTCC" => SetTarget::LastCC,
                    _ => {
                        return Command::Error(ParseErrorNode {
                            code: "IDC0001E".to_string(),
                            message: format!("SET requires LASTCC or MAXCC, got: {kw}"),
                            position: self.pos,
                        });
                    }
                };
                let value = self.read_paren_number().unwrap_or(0) as u8;
                Command::Set(SetCommand {
                    target,
                    value: value.min(16),
                })
            }
            _ => {
                let pos = self.pos;
                self.skip_to_next_command();
                Command::Error(ParseErrorNode {
                    code: "IDC0001E".to_string(),
                    message: "SET requires LASTCC or MAXCC".to_string(),
                    position: pos,
                })
            }
        }
    }

    fn parse_if(&mut self) -> Command {
        let condition = self.parse_condition();

        // Expect THEN
        let then_commands = if matches!(self.peek(), Token::Keyword(ref k) if k == "THEN") {
            self.advance();
            self.parse_command_block()
        } else {
            vec![]
        };

        // Optional ELSE
        let else_commands = if matches!(self.peek(), Token::Keyword(ref k) if k == "ELSE") {
            self.advance();
            Some(self.parse_command_block())
        } else {
            None
        };

        Command::If(IfCommand {
            condition,
            then_commands,
            else_commands,
        })
    }

    fn parse_condition(&mut self) -> Condition {
        let left = self.parse_simple_condition();

        // Check for AND/OR
        match self.peek() {
            Token::LogicalOp(op) => {
                let op = *op;
                self.advance();
                let right = self.parse_condition();
                match op {
                    token::LogOp::And => Condition::And(Box::new(left), Box::new(right)),
                    token::LogOp::Or => Condition::Or(Box::new(left), Box::new(right)),
                }
            }
            _ => left,
        }
    }

    fn parse_simple_condition(&mut self) -> Condition {
        // LASTCC/MAXCC op value
        let register = match self.peek().clone() {
            Token::Keyword(ref kw) if kw == "LASTCC" => {
                self.advance();
                ConditionRegister::LastCC
            }
            Token::Keyword(ref kw) if kw == "MAXCC" => {
                self.advance();
                ConditionRegister::MaxCC
            }
            Token::OpenParen => {
                self.advance();
                let cond = self.parse_condition();
                self.expect_close_paren();
                return cond;
            }
            _ => {
                // Invalid register — skip and return a default
                self.advance();
                return Condition::Compare {
                    register: ConditionRegister::LastCC,
                    op: CmpOp::Eq,
                    value: 0,
                };
            }
        };

        let op = match self.peek() {
            Token::CompareOp(op) => {
                let op = *op;
                self.advance();
                op
            }
            _ => CmpOp::Eq,
        };

        let value = self.read_number().unwrap_or(0) as u8;

        Condition::Compare {
            register,
            op,
            value,
        }
    }

    fn parse_command_block(&mut self) -> Vec<Command> {
        // Check for DO/END block
        if matches!(self.peek(), Token::Keyword(ref k) if k == "DO") {
            self.advance();
            let mut commands = Vec::new();
            while !matches!(self.peek(), Token::Keyword(ref k) if k == "END") {
                if self.at_end() {
                    break;
                }
                commands.push(self.parse_command());
                self.skip_semicolons();
            }
            if matches!(self.peek(), Token::Keyword(ref k) if k == "END") {
                self.advance();
            }
            commands
        } else {
            // Single command
            vec![self.parse_command()]
        }
    }
}
