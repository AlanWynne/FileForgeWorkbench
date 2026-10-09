//! Window_Band: the physically-resident record-range cache (CR-CH-058 F2).
//!
//! The Window_Band models WHICH records' bytes are resident, as a range of
//! RECORD numbers (not bytes), decoupled from the logical index's
//! `Total_Records` (CR-CH-058 Req 12.4). It is a 3-page band -- the current page
//! plus one prefetched page above and one below -- where a PAGE is the maximum
//! records that fit a full screen at the SMALLEST zoom (so zoom-in never needs
//! more than the resident band; zoom is render-only, Req 12.6, wired shell-side).
//!
//! CRITICAL (Req 12.5): `total_records` (the whole-file authority, from the
//! index) and the resident band span are SEPARATE values and are NEVER fused.
//! A scrollbar/navigation consumer sizes on `total_records()`, never on the
//! band -- that is the fix for the owner-observed "scrollbar collapses to the
//! window buffer" bug.
//!
//! This module is a PURE document-model primitive: it computes which record
//! range should be resident and when the band must shift (load/evict), with
//! hysteresis so a fast jump performs ONE shift rather than a cascade
//! (Req 12.7), and resolves a `down N`/`up N` jump by recentring on the target
//! without touching intermediate records (Req 12.8). The actual byte read/evict
//! and the egui scrollbar/zoom wiring live in the shell/viewport layer (F2
//! shell slice) and consume this primitive; nothing here depends on egui or the
//! shell.

use crate::types::RecordNumber;

/// An inclusive-start, exclusive-end range of record numbers `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRange {
    /// First resident record (inclusive).
    pub start: u64,
    /// One past the last resident record (exclusive).
    pub end: u64,
}

impl RecordRange {
    /// An empty range at `start`.
    pub fn empty(start: u64) -> Self {
        Self { start, end: start }
    }

    /// Number of records in the range.
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Whether `record` is within `[start, end)`.
    pub fn contains(&self, record: u64) -> bool {
        record >= self.start && record < self.end
    }
}

/// The 3-page resident record band over a document of `total_records` records.
///
/// The band is `[band.start, band.end)` in RECORD numbers; it is kept to at most
/// three pages (prev + current + next), clamped to `[0, total_records)`. The
/// band shifts only when the viewport's current page leaves the resident band
/// (hysteresis), so small scrolling within the band triggers no load/evict.
#[derive(Debug, Clone)]
pub struct WindowBand {
    /// Whole-file record count from the index -- the navigation/scrollbar
    /// authority (Req 12.5). NEVER equal to the band span except for a document
    /// that fits in the band.
    total_records: u64,
    /// Records per page (max records on a full screen at the SMALLEST zoom).
    page_size: u64,
    /// The current viewport page index (0-based): the page the user is viewing.
    current_page: u64,
    /// The resident record range `[start, end)` (<= 3 pages), clamped to the doc.
    band: RecordRange,
}

impl WindowBand {
    /// Create a band for a document of `total_records`, `page_size` records per
    /// page, positioned at page 0 (top). `page_size` is clamped to at least 1.
    pub fn new(total_records: u64, page_size: u64) -> Self {
        let page_size = page_size.max(1);
        let mut band = Self {
            total_records,
            page_size,
            current_page: 0,
            band: RecordRange::empty(0),
        };
        band.band = band.compute_band(0);
        band
    }

    /// The whole-file record count (the authority). NEVER the resident span.
    ///
    /// Validates: Requirement 12.5
    pub fn total_records(&self) -> u64 {
        self.total_records
    }

    /// Records per page (smallest-zoom full-screen capacity).
    pub fn page_size(&self) -> u64 {
        self.page_size
    }

    /// The current 0-based viewport page.
    pub fn current_page(&self) -> u64 {
        self.current_page
    }

    /// The resident record range `[start, end)`. This is the Window_Band span --
    /// DISTINCT from `total_records()` (Req 12.5), at most 3 pages (Req 12.4).
    pub fn resident_range(&self) -> RecordRange {
        self.band
    }

    /// Number of resident records (the band span). NEVER use this as the
    /// scrollbar/navigation total -- use `total_records()` for that (Req 12.5).
    pub fn resident_len(&self) -> u64 {
        self.band.len()
    }

    /// Update the whole-file total (e.g. after an edit changes record count or
    /// a background index scan refines an estimate). Recomputes the band around
    /// the current page clamped to the new total; does NOT move the current page
    /// except to clamp it into range.
    pub fn set_total_records(&mut self, total: u64) {
        self.total_records = total;
        let last_page = self.last_page();
        if self.current_page > last_page {
            self.current_page = last_page;
        }
        self.band = self.compute_band(self.current_page);
    }

    /// The last valid page index for the current total (0 when empty).
    fn last_page(&self) -> u64 {
        if self.total_records == 0 {
            0
        } else {
            (self.total_records - 1) / self.page_size
        }
    }

    /// The 0-based page that record `n` falls on.
    pub fn page_of(&self, n: RecordNumber) -> u64 {
        n.0 / self.page_size
    }

    /// Compute the 3-page band `[start, end)` centred on `page`, clamped to the
    /// document. Prev page, current page, next page; a `page` at the document
    /// edge yields a 2-page band (no wraparound, no over-read).
    fn compute_band(&self, page: u64) -> RecordRange {
        if self.total_records == 0 {
            return RecordRange::empty(0);
        }
        let start_page = page.saturating_sub(1);
        let end_page = page + 1; // inclusive page index of the last band page
        let start = start_page * self.page_size;
        let end = ((end_page + 1) * self.page_size).min(self.total_records);
        RecordRange { start, end }
    }

    /// Scroll the viewport so the current page becomes the page containing
    /// `top_record`, shifting (loading/evicting) the band ONLY if that page is
    /// not already covered by the resident band (HYSTERESIS, Req 12.7). Returns
    /// `true` if the band shifted (a load/evict is required), `false` if the
    /// move stayed within the resident band (no I/O).
    ///
    /// Validates: Requirement 12.7
    pub fn scroll_to_record(&mut self, top_record: RecordNumber) -> bool {
        let target_page = self.page_of(top_record).min(self.last_page());
        self.current_page = target_page;
        // Hysteresis: only shift the band if the target page's full page range
        // is not already fully resident. A page is "covered" when both its first
        // and (clamped) last record are inside the current band.
        let page_start = target_page * self.page_size;
        let page_end = ((target_page + 1) * self.page_size).min(self.total_records);
        let covered = self.band.contains(page_start)
            && (page_end == page_start || self.band.contains(page_end - 1));
        if covered {
            return false;
        }
        self.band = self.compute_band(target_page);
        true
    }

    /// Jump directly to the page containing `target` (a `down N`/`up N` jump that
    /// may land far outside the resident band), loading that band in ONE shift
    /// and reading NO intermediate records -- the target is resolved via its
    /// record number, the band recomputed around it, nothing between the old and
    /// new band is touched (Req 12.8). Always performs one shift (returns the new
    /// resident range).
    ///
    /// Validates: Requirement 12.8
    pub fn jump_to_record(&mut self, target: RecordNumber) -> RecordRange {
        let target_page = self.page_of(target).min(self.last_page());
        self.current_page = target_page;
        self.band = self.compute_band(target_page);
        self.band
    }

    /// Records that must remain resident regardless of the band because they are
    /// DIRTY (edited, not yet saved) -- the caller supplies them and this returns
    /// the eviction set = (previously resident) MINUS (new band) MINUS (pinned).
    /// Pinned dirty records are NEVER evicted (Req 12.7). This is a pure set
    /// computation the shell's byte-cache uses when the band shifts.
    ///
    /// Validates: Requirement 12.7 (dirty pieces pinned)
    pub fn evictable(&self, previous: RecordRange, pinned: &[RecordNumber]) -> Vec<RecordNumber> {
        let mut out = Vec::new();
        for r in previous.start..previous.end {
            if self.band.contains(r) {
                continue; // still resident in the new band
            }
            if pinned.iter().any(|p| p.0 == r) {
                continue; // dirty -> pinned, never evicted
            }
            out.push(RecordNumber(r));
        }
        out
    }
}

#[cfg(test)]
#[path = "window_band_tests.rs"]
mod tests;
