//! # Message routing and pre-action capture state (CR-NR-115 follow-up)
//!
//! Two small, additive concerns of the common frame:
//!
//! * ROUTING (workspace-framework Req 7.28): INFORMATIONAL / confirmation
//!   messages go to the Message_Line band (`message_line`) via
//!   [`WorkbenchShell::post_info`]; ERRORS stay on the Status_Bar error slot
//!   (`open_error`). Classification rule: a message is INFO when the command
//!   SUCCEEDED and the text reports a result; it is an ERROR when it reports a
//!   failure, refusal, bad argument or a "not available" stub.
//! * CAPTURE TIMING (Req 7.29, screen-snapshot-scrm 30.28): a SNAPSHOT/CAPTURE
//!   shows the frame as it was when the command was issued. The per-action seam
//!   `note_action_dispatched` stashes a [`PreActionFrame`] (IDEMPOTENT per
//!   action, because the key-forward path invokes the seam twice) BEFORE it
//!   clears the Message_Line and restamps `Last:`; the outermost action entry
//!   point drops the stash again when the action ends. `active_screen_capture`
//!   remains the one capture seam and reads through the helpers below.
//!
//! Validates: workspace-framework Requirement 7.28, 7.29.

use super::WorkbenchShell;

/// The slice of common-frame state a capture must reproduce from BEFORE the
/// triggering action ran (the Message_Line and the `Last:` stamp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreActionFrame {
    /// The Message_Line contents present before the action cleared them.
    pub(crate) message_line: Vec<String>,
    /// The last-action time present before the action restamped it.
    pub(crate) last_action_at: chrono::DateTime<chrono::Local>,
    /// The `Command ===>` field as it was BEFORE the triggering command
    /// (screen-snapshot-scrm Req 30.24(b)). Key-forward actions keep the text
    /// the user typed; a typed command line clears it (the field IS the command).
    pub(crate) command_text: String,
}

/// How the clipboard placement of a composite snapshot ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SnapshotPlaceOutcome {
    /// Both the picture and the text flavor were offered.
    Both,
    /// Only one flavor was offered (still a success for what landed).
    Partial,
    /// The clipboard could not be used (an ERROR).
    Failed,
}

impl WorkbenchShell {
    /// Post an INFORMATIONAL / confirmation message to the Message_Line band
    /// (Req 7.28). Messages live until the next action clears them (Req 7.12).
    pub(crate) fn post_info(&mut self, text: impl Into<String>) {
        self.message_line.push(text.into());
    }

    /// Route the result of placing the composite snapshot on the clipboard:
    /// success to the Message_Line, failure to the Status_Bar error slot.
    pub(crate) fn post_snapshot_place_outcome(&mut self, outcome: SnapshotPlaceOutcome, fmt: &str) {
        let args = ff_i18n::MessageArgs::new().set("format", fmt.to_string());
        match outcome {
            SnapshotPlaceOutcome::Both => self.post_info(ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-both-copied",
                &args,
            )),
            SnapshotPlaceOutcome::Partial => self.post_info(ff_i18n::t_args(
                "ff-desktop-shell-msg-scrm-snapshot-copied",
                &args,
            )),
            SnapshotPlaceOutcome::Failed => {
                self.open_error = Some(ff_i18n::t_args(
                    "ff-desktop-shell-msg-scrm-snapshot-no-clipboard",
                    &args,
                ));
            }
        }
    }

    /// Stash the pre-action state ONCE per action (idempotent: a second call
    /// within the same action keeps the first stash).
    pub(super) fn stash_pre_action_frame(&mut self) {
        if self.pre_action_frame.is_none() {
            self.pre_action_frame = Some(PreActionFrame {
                message_line: self.message_line.clone(),
                last_action_at: self.last_action_at,
                command_text: self.command_text.clone(),
            });
        }
    }

    /// The `Command ===>` text a capture shows: the pre-action field while an
    /// action is in flight, else empty (Req 30.24(b)).
    pub(super) fn frame_command_text(&self) -> String {
        match &self.pre_action_frame {
            Some(pre) => pre.command_text.clone(),
            None => String::new(),
        }
    }

    /// The Message_Line entries a capture shows: the pre-action stash while an
    /// action is in flight, else the live entries.
    pub(super) fn frame_message_lines(&self) -> &[String] {
        match &self.pre_action_frame {
            Some(pre) => &pre.message_line,
            None => &self.message_line,
        }
    }

    /// The `Last:` stamp a capture shows (pre-action while an action is in flight).
    pub(super) fn frame_last_action_at(&self) -> chrono::DateTime<chrono::Local> {
        match &self.pre_action_frame {
            Some(pre) => pre.last_action_at,
            None => self.last_action_at,
        }
    }
}
