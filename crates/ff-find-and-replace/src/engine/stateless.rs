use crate::case_folder::CaseFolder;
use crate::direction::SearchDirection;
use crate::error::FindReplaceError;
use crate::hex_search::parse_hex_pattern;
use crate::indexer::CharacterIndexer;
use crate::literal;
use crate::request::FindRequest;
use crate::result::FindOutcome;
use crate::scope::{resolve_column_range, Bounds, ScopeFilterProvider};
use crate::search_mode::SearchMode;
use crate::types::BytePosition;

use super::FindEngineConfig;

/// Stateless find execution (for find_for_filter and change_all internal use).
pub(super) fn execute_find_stateless(
    request: &FindRequest,
    indexer: &dyn CharacterIndexer,
    _scope_filter: &dyn ScopeFilterProvider,
    bounds: Option<&Bounds>,
    config: &FindEngineConfig,
    case_folder: &CaseFolder,
) -> Result<FindOutcome, FindReplaceError> {
    let _col_range = resolve_column_range(
        request.column_range.as_ref(),
        bounds,
        config.bounds_affect_find,
    );

    let (start, end) = match request.direction {
        SearchDirection::Next => (request.cursor_position, BytePosition(indexer.length())),
        SearchDirection::Prev => (BytePosition::ZERO, request.cursor_position),
        SearchDirection::First => (BytePosition::ZERO, BytePosition(indexer.length())),
        SearchDirection::Last => (BytePosition::ZERO, BytePosition(indexer.length())),
    };

    match request.mode {
        SearchMode::Literal => {
            let pattern = request.term.as_bytes();
            if request.case_sensitive {
                match request.direction {
                    SearchDirection::Next | SearchDirection::First => {
                        match literal::find_literal_forward(
                            pattern,
                            indexer,
                            start,
                            end,
                            request.word_match,
                        ) {
                            Some(r) => Ok(FindOutcome::Found(r)),
                            None => Ok(FindOutcome::NotFound {
                                term: request.term.clone(),
                            }),
                        }
                    }
                    SearchDirection::Prev | SearchDirection::Last => {
                        let from = if request.direction == SearchDirection::Last {
                            BytePosition(indexer.length())
                        } else {
                            end
                        };
                        match literal::find_literal_backward(
                            pattern,
                            indexer,
                            from,
                            start,
                            request.word_match,
                        ) {
                            Some(r) => Ok(FindOutcome::Found(r)),
                            None => Ok(FindOutcome::NotFound {
                                term: request.term.clone(),
                            }),
                        }
                    }
                }
            } else {
                let folded = case_folder.fold_bytes(pattern);
                match request.direction {
                    SearchDirection::Next | SearchDirection::First => {
                        match literal::find_literal_case_insensitive_forward(
                            &folded,
                            indexer,
                            start,
                            end,
                            case_folder,
                            request.word_match,
                        ) {
                            Some(r) => Ok(FindOutcome::Found(r)),
                            None => Ok(FindOutcome::NotFound {
                                term: request.term.clone(),
                            }),
                        }
                    }
                    SearchDirection::Prev | SearchDirection::Last => {
                        let from = if request.direction == SearchDirection::Last {
                            BytePosition(indexer.length())
                        } else {
                            end
                        };
                        match literal::find_literal_case_insensitive_backward(
                            &folded,
                            indexer,
                            from,
                            start,
                            BytePosition(indexer.length()),
                            case_folder,
                            request.word_match,
                        ) {
                            Some(r) => Ok(FindOutcome::Found(r)),
                            None => Ok(FindOutcome::NotFound {
                                term: request.term.clone(),
                            }),
                        }
                    }
                }
            }
        }
        SearchMode::HexBytes => {
            let pattern = parse_hex_pattern(&request.term)?;
            match literal::find_literal_forward(&pattern, indexer, start, end, request.word_match) {
                Some(r) => Ok(FindOutcome::Found(r)),
                None => Ok(FindOutcome::NotFound {
                    term: request.term.clone(),
                }),
            }
        }
        SearchMode::Regex => {
            // Stateless regex execution is limited
            Ok(FindOutcome::NotFound {
                term: request.term.clone(),
            })
        }
    }
}
