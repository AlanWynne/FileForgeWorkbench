//! Tests for `WindowBand` (CR-CH-058 F2, document-model slice).

use super::*;
use crate::types::RecordNumber;

fn r(n: u64) -> RecordNumber {
    RecordNumber(n)
}

#[test]
fn new_band_is_at_most_three_pages_at_top() {
    // Validates: Requirement 12.4 -- page 0 at the top yields a 2-page band
    // (current + next; no page above the top), 10 records/page.
    let band = WindowBand::new(1000, 10);
    let range = band.resident_range();
    assert_eq!(range.start, 0, "top band starts at record 0");
    assert_eq!(
        range.end, 20,
        "top band covers current + next page = 2 pages"
    );
    assert_eq!(band.current_page(), 0);
}

#[test]
fn middle_page_band_is_three_pages() {
    // Validates: Requirement 12.4 -- a mid-document page yields prev+cur+next.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(55)); // page 5
    let range = band.resident_range();
    assert_eq!(band.current_page(), 5);
    assert_eq!(range.start, 40, "band starts one page above (page 4)");
    assert_eq!(range.end, 70, "band ends one page below (through page 6)");
    assert_eq!(range.len(), 30, "three pages resident");
}

#[test]
fn band_clamps_at_document_end() {
    // Validates: Requirement 12.4 -- the last page's band does not over-read.
    let mut band = WindowBand::new(95, 10); // last page = 9 (records 90..95)
    band.jump_to_record(r(94));
    let range = band.resident_range();
    assert_eq!(band.current_page(), 9);
    assert_eq!(range.start, 80, "prev page 8");
    assert_eq!(range.end, 95, "clamped to total, not 100");
}

#[test]
fn total_records_is_distinct_from_resident_span() {
    // Validates: Requirement 12.5 (THE bug fix) -- total_records (whole file) is
    // NEVER the resident band span. A huge file has a tiny band but a huge total.
    let band = WindowBand::new(100_000_000, 50);
    assert_eq!(
        band.total_records(),
        100_000_000,
        "scrollbar authority = whole file"
    );
    assert!(
        band.resident_len() <= 150,
        "resident span is at most 3 pages (150), NOT the 100M total"
    );
    assert_ne!(
        band.total_records(),
        band.resident_len(),
        "total and resident span must never be fused"
    );
}

#[test]
fn small_document_band_equals_total_but_values_stay_separate() {
    // Validates: Requirement 12.5 -- even when the whole doc fits in the band,
    // total_records() and resident_len() are separate accessors (a scrollbar
    // sizing on total is still correct; it just happens to equal the span here).
    let band = WindowBand::new(7, 100); // 7 records, 100/page -> all resident
    assert_eq!(band.total_records(), 7);
    assert_eq!(band.resident_range(), RecordRange { start: 0, end: 7 });
    assert_eq!(band.resident_len(), 7);
}

#[test]
fn scroll_within_band_does_not_shift() {
    // Validates: Requirement 12.7 -- hysteresis: scrolling to a record already
    // within the resident band performs NO load/evict.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(55)); // band [40,70), page 5
    let before = band.resident_range();
    // Scroll to a record on page 4 (record 42) -- still inside [40,70).
    let shifted = band.scroll_to_record(r(42));
    assert!(
        !shifted,
        "scrolling within the resident band must not shift"
    );
    assert_eq!(band.resident_range(), before, "band unchanged");
}

#[test]
fn scroll_past_band_edge_shifts_once() {
    // Validates: Requirement 12.7 -- leaving the band triggers exactly one shift.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(55)); // band [40,70)
                                // Scroll to page 8 (record 80) -- outside [40,70) -> one shift.
    let shifted = band.scroll_to_record(r(80));
    assert!(shifted, "leaving the band shifts");
    assert_eq!(band.current_page(), 8);
    assert_eq!(
        band.resident_range(),
        RecordRange {
            start: 70,
            end: 100
        }
    );
}

#[test]
fn fast_jump_is_a_single_shift_not_a_cascade() {
    // Validates: Requirement 12.7, 12.8 -- a far jump (down 9999-style) is ONE
    // band recompute around the target; it does not walk page-by-page.
    let mut band = WindowBand::new(1_000_000, 10);
    band.jump_to_record(r(0)); // top
    let range = band.jump_to_record(r(500_000)); // jump far
    assert_eq!(band.current_page(), 50_000);
    assert_eq!(
        range.start, 499_990,
        "band recomputed directly around the target"
    );
    assert_eq!(range.end, 500_020);
    // No assertion about intermediate reads is needed: jump_to_record computes
    // the band arithmetically from the target page and never iterates the
    // records between the old and new band (Req 12.8). The band jumped straight
    // from [0,20) to [499990,500020) with no intervening page.
}

#[test]
fn jump_up_resolves_target_without_intermediates() {
    // Validates: Requirement 12.8 -- an up jump lands directly on the target.
    let mut band = WindowBand::new(1_000_000, 10);
    band.jump_to_record(r(900_000));
    let range = band.jump_to_record(r(10)); // jump back up to near the top
    assert_eq!(band.current_page(), 1);
    assert_eq!(range.start, 0, "page 0 (prev of page 1 clamps to 0)");
    assert_eq!(range.end, 30);
}

#[test]
fn dirty_pinned_records_are_never_evicted() {
    // Validates: Requirement 12.7 -- dirty (edited) records stay resident even
    // when the band shifts away from them.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(5)); // band [0,20)
    let previous = band.resident_range();
    // Pin record 3 (dirty), then jump far away so the band no longer covers it.
    let pinned = [r(3)];
    band.jump_to_record(r(500)); // band moves to [490,520)
    let evicted = band.evictable(previous, &pinned);
    assert!(
        !evicted.iter().any(|e| e.0 == 3),
        "pinned dirty record 3 must NOT be in the eviction set"
    );
    // A non-pinned previously-resident record (e.g. 7) IS evictable.
    assert!(
        evicted.iter().any(|e| e.0 == 7),
        "unpinned record 7 is evictable"
    );
}

#[test]
fn evictable_excludes_records_still_in_the_new_band() {
    // Validates: Requirement 12.7 -- records that remain resident after a small
    // shift are not evicted.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(55)); // [40,70)
    let previous = band.resident_range();
    band.scroll_to_record(r(80)); // shift to [70,100)
    let evicted = band.evictable(previous, &[]);
    // Records 40..70 were resident; none of them are in [70,100), so all evict.
    assert!(evicted.iter().all(|e| e.0 >= 40 && e.0 < 70));
    assert_eq!(evicted.len(), 30);
}

#[test]
fn set_total_records_reclamps_without_moving_page_when_in_range() {
    // Validates: Requirement 12.5 -- refining the total (estimate -> exact, or
    // after an edit) updates the authority and reclamps the band; a current page
    // still in range is preserved.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(55)); // page 5
    band.set_total_records(2000);
    assert_eq!(band.total_records(), 2000);
    assert_eq!(band.current_page(), 5, "page preserved when still in range");
    assert_eq!(band.resident_range(), RecordRange { start: 40, end: 70 });
}

#[test]
fn set_total_records_clamps_current_page_when_shrunk() {
    // Validates: Requirement 12.5 -- shrinking the total below the current page
    // clamps the page into range.
    let mut band = WindowBand::new(1000, 10);
    band.jump_to_record(r(950)); // page 95
    band.set_total_records(100); // last page now 9
    assert_eq!(band.total_records(), 100);
    assert_eq!(band.current_page(), 9, "page clamped to the new last page");
    assert_eq!(
        band.resident_range(),
        RecordRange {
            start: 80,
            end: 100
        }
    );
}

#[test]
fn empty_document_has_empty_band() {
    // Validates: Requirement 12.4 -- a 0-record document yields an empty band.
    let band = WindowBand::new(0, 10);
    assert_eq!(band.total_records(), 0);
    assert!(band.resident_range().is_empty());
}

#[test]
fn page_of_maps_records_to_pages() {
    let band = WindowBand::new(1000, 10);
    assert_eq!(band.page_of(r(0)), 0);
    assert_eq!(band.page_of(r(9)), 0);
    assert_eq!(band.page_of(r(10)), 1);
    assert_eq!(band.page_of(r(55)), 5);
}
