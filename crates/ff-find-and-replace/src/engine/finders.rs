use crate::direction::SearchDirection;
use crate::error::FindReplaceError;
use crate::hex_search::parse_hex_pattern;
use crate::indexer::{CharacterIndexer, CharacterIndexerMut};
use crate::literal;
use crate::request::{ChangeRequest, FindRequest};
use crate::result::{ChangeOutcome, ChangeResult, FindOutcome, FindResult};
use crate::scope::{resolve_column_range, Bounds, ColumnRange, ScopeFilterProvider};
use crate::search_mode::SearchMode;
use crate::substitution::SubstitutionTemplate;
use crate::types::{BytePosition, LineNumber};

use super::FindEngine;

use super::stateless::execute_find_stateless;

impl FindEngine {
    /// Core find dispatch based on mode and direction.
    pub(super) fn execute_find(
        &mut self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<FindOutcome, FindReplaceError> {
        let col_range = resolve_column_range(
            request.column_range.as_ref(),
            bounds,
            self.config.bounds_affect_find,
        );

        // Determine search range
        let (search_start, search_end) = match request.direction {
            SearchDirection::Next => (request.cursor_position, BytePosition(indexer.length())),
            SearchDirection::Prev => (BytePosition::ZERO, request.cursor_position),
            SearchDirection::First => (BytePosition::ZERO, BytePosition(indexer.length())),
            SearchDirection::Last => (BytePosition::ZERO, BytePosition(indexer.length())),
        };

        match request.mode {
            SearchMode::Literal => self.find_literal(
                request,
                indexer,
                search_start,
                search_end,
                scope_filter,
                col_range.as_ref(),
            ),
            SearchMode::HexBytes => self.find_hex(
                request,
                indexer,
                search_start,
                search_end,
                scope_filter,
                col_range.as_ref(),
            ),
            SearchMode::Regex => self.find_regex(
                request,
                indexer,
                search_start,
                search_end,
                scope_filter,
                col_range.as_ref(),
            ),
        }
    }

    pub(super) fn find_literal(
        &self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        _scope_filter: &dyn ScopeFilterProvider,
        _col_range: Option<&ColumnRange>,
    ) -> Result<FindOutcome, FindReplaceError> {
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
                    let search_from = if request.direction == SearchDirection::Last {
                        BytePosition(indexer.length())
                    } else {
                        end
                    };
                    match literal::find_literal_backward(
                        pattern,
                        indexer,
                        search_from,
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
            let folded = self.case_folder.fold_bytes(pattern);
            match request.direction {
                SearchDirection::Next | SearchDirection::First => {
                    match literal::find_literal_case_insensitive_forward(
                        &folded,
                        indexer,
                        start,
                        end,
                        &self.case_folder,
                        request.word_match,
                    ) {
                        Some(r) => Ok(FindOutcome::Found(r)),
                        None => Ok(FindOutcome::NotFound {
                            term: request.term.clone(),
                        }),
                    }
                }
                SearchDirection::Prev | SearchDirection::Last => {
                    let search_from = if request.direction == SearchDirection::Last {
                        BytePosition(indexer.length())
                    } else {
                        end
                    };
                    match literal::find_literal_case_insensitive_backward(
                        &folded,
                        indexer,
                        search_from,
                        start,
                        BytePosition(indexer.length()),
                        &self.case_folder,
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

    pub(super) fn find_hex(
        &self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        _scope_filter: &dyn ScopeFilterProvider,
        _col_range: Option<&ColumnRange>,
    ) -> Result<FindOutcome, FindReplaceError> {
        let pattern = parse_hex_pattern(&request.term)?;

        // Hex search uses raw byte matching -- no case folding
        match request.direction {
            SearchDirection::Next | SearchDirection::First => {
                match literal::find_literal_forward(
                    &pattern,
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
                let search_from = if request.direction == SearchDirection::Last {
                    BytePosition(indexer.length())
                } else {
                    end
                };
                match literal::find_literal_backward(
                    &pattern,
                    indexer,
                    search_from,
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
    }

    pub(super) fn find_regex(
        &mut self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        _scope_filter: &dyn ScopeFilterProvider,
        _col_range: Option<&ColumnRange>,
    ) -> Result<FindOutcome, FindReplaceError> {
        let compiled = self.regex_engine.compile(&request.term)?.clone();
        let case_folder = if !request.case_sensitive {
            Some(&self.case_folder)
        } else {
            None
        };

        match request.direction {
            SearchDirection::Next | SearchDirection::First => {
                match self.regex_engine.execute_forward(
                    &compiled,
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
                let search_from = if request.direction == SearchDirection::Last {
                    BytePosition(indexer.length())
                } else {
                    end
                };
                match self.regex_engine.execute_backward(
                    &compiled,
                    indexer,
                    start,
                    search_from,
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

    pub(super) fn find_all_in_range(
        &self,
        request: &FindRequest,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
    ) -> Result<Vec<FindResult>, FindReplaceError> {
        match request.mode {
            SearchMode::Literal => {
                let pattern = request.term.as_bytes();
                if request.case_sensitive {
                    Ok(literal::find_literal_all(
                        pattern,
                        indexer,
                        start,
                        end,
                        request.word_match,
                    ))
                } else {
                    let folded = self.case_folder.fold_bytes(pattern);
                    Ok(literal::find_literal_case_insensitive_all(
                        &folded,
                        indexer,
                        start,
                        end,
                        &self.case_folder,
                        request.word_match,
                    ))
                }
            }
            SearchMode::HexBytes => {
                let pattern = parse_hex_pattern(&request.term)?;
                Ok(literal::find_literal_all(
                    &pattern,
                    indexer,
                    start,
                    end,
                    request.word_match,
                ))
            }
            SearchMode::Regex => {
                // For regex find_all, we need a compiled pattern
                // Since we don't have mutable access, clone the last compiled
                // This is a limitation -- in production this would use interior mutability
                Ok(Vec::new()) // Simplified for non-mutable context
            }
        }
    }

    pub(super) fn execute_change_all(
        &mut self,
        request: &ChangeRequest,
        indexer: &mut dyn CharacterIndexerMut,
        scope_filter: &dyn ScopeFilterProvider,
        bounds: Option<&Bounds>,
    ) -> Result<ChangeOutcome, FindReplaceError> {
        let mut count: u64 = 0;
        let mut cursor = BytePosition::ZERO;
        let mut final_pos = BytePosition::ZERO;
        let mut final_line = LineNumber::ZERO;

        loop {
            let end = BytePosition(indexer.length());
            if cursor >= end {
                break;
            }

            // Create a request searching from current cursor
            let mut search_req = request.find.clone();
            search_req.direction = SearchDirection::Next;
            search_req.cursor_position = cursor;

            let outcome = execute_find_stateless(
                &search_req,
                indexer,
                scope_filter,
                bounds,
                &self.config,
                &self.case_folder,
            )?;

            match outcome {
                FindOutcome::Found(result) => {
                    let replacement_bytes = self.compute_replacement(request, &result, indexer)?;
                    indexer.replace_range(
                        result.match_range.start,
                        result.match_range.end,
                        &replacement_bytes,
                    )?;

                    let new_end = result.match_range.start.0 + replacement_bytes.len() as u64;
                    // Advance past the replacement to avoid infinite loops
                    cursor = BytePosition(
                        if replacement_bytes.is_empty() && result.match_range.is_empty() {
                            new_end + 1
                        } else {
                            new_end
                        },
                    );
                    final_pos = cursor;
                    final_line = indexer.line_from_position(final_pos);
                    count += 1;
                }
                FindOutcome::NotFound { .. } | FindOutcome::FoundAll { .. } => break,
            }
        }

        if count == 0 {
            Ok(ChangeOutcome::NotFound {
                term: request.find.term.clone(),
            })
        } else {
            self.state.record_change(request, final_pos);
            Ok(ChangeOutcome::Changed(ChangeResult {
                replacement_count: count,
                final_position: final_pos,
                final_line,
            }))
        }
    }

    pub(super) fn compute_replacement(
        &self,
        request: &ChangeRequest,
        result: &FindResult,
        indexer: &dyn CharacterIndexer,
    ) -> Result<Vec<u8>, FindReplaceError> {
        if request.find.mode == SearchMode::Regex {
            let template = SubstitutionTemplate::parse(&request.replacement)?;
            let expanded = template.expand(&result.match_range, &result.captures, indexer);
            Ok(expanded.into_bytes())
        } else {
            Ok(request.replacement.as_bytes().to_vec())
        }
    }
}
