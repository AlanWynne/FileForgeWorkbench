//! Zoom common status segment tests (CR-NR-115 follow-up, workspace-framework
//! Requirement 7.16-7.21 revised, screen-snapshot-scrm Requirement 30.10): zoom is
//! a COMMON status segment after Prof, ALWAYS shown, as `Zoom: 0`, `Zoom: +14`,
//! `Zoom: -3` (negative zoom is supported: default min_offset is -10).

use super::tests_common::*;

fn shell_with_zoom(offset: i32) -> super::WorkbenchShell {
    super::i18n_bootstrap::install_catalogue_for_tests();
    ff_session::set_active_profile(None);
    let mut shell = make_shell();
    shell.zoom.set_offset(offset);
    shell
}

// Validates: workspace-framework Requirement 7.16 -- zero zoom is shown.
#[test]
fn zoom_label_zero_is_zoom_colon_0() {
    assert_eq!(shell_with_zoom(0).format_zoom_label(), "Zoom: 0");
}

// Validates: workspace-framework Requirement 7.16 -- positive zoom has a sign.
#[test]
fn zoom_label_positive_has_plus_sign() {
    assert_eq!(shell_with_zoom(14).format_zoom_label(), "Zoom: +14");
}

// Validates: workspace-framework Requirement 7.16 -- negative zoom is supported.
#[test]
fn zoom_label_negative_has_minus_sign() {
    assert_eq!(shell_with_zoom(-3).format_zoom_label(), "Zoom: -3");
}

// Validates: workspace-framework Requirement 7.16, 7.17 and screen-snapshot-scrm
// Requirement 30.10 -- the status text is Last | Prof | Zoom, then the Kind cells.
#[test]
fn status_band_order_is_last_prof_zoom_then_kind_cells() {
    let mut shell = shell_with_zoom(0);
    shell.shell_new_untitled();
    shell.last_action_at = super::tests_common_frame_golden::fixed_stamp();
    let mut model = ff_screen_model::ScreenModel::new("Untitled");
    shell.apply_common_frame(&mut model);
    assert_eq!(
        model.status_bar.text,
        "Last: 11:02 | Prof: default | Zoom: 0 | Ln 1, Col 1 | UTF-8 | 1 lines"
    );
}

// Validates: screen-snapshot-scrm Requirement 30.10 -- the snapshot band carries
// exactly the label the live bar draws (one formatter), for +14 and -3.
#[test]
fn snapshot_band_uses_the_live_formatter() {
    for offset in [14, -3] {
        let shell = shell_with_zoom(offset);
        let mut model = ff_screen_model::ScreenModel::new("T");
        shell.apply_common_frame(&mut model);
        assert!(
            model.status_bar.text.contains(&shell.format_zoom_label()),
            "{}",
            model.status_bar.text
        );
    }
}

// Validates: workspace-framework Requirement 7.16 -- the LIVE status bar draws
// the zoom label (always, also at 0), and it is not a Tab stop (B055): nothing
// in the status bar takes keyboard focus.
// The hover tooltip text is MANUAL: kittest cannot read hover popups headlessly.
#[test]
fn live_status_bar_shows_zoom_label_and_is_not_a_tab_stop() {
    use egui_kittest::kittest::Queryable;
    for (offset, label) in [(0, "Zoom: 0"), (14, "Zoom: +14"), (-3, "Zoom: -3")] {
        let mut shell = shell_with_zoom(offset);
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(1200.0, 300.0))
            .build(move |ctx| {
                shell.render_status_bar(ctx);
            });
        harness.run();
        assert_eq!(harness.query_all_by_label(label).count(), 1, "{label}");
        harness.key_press(egui::Key::Tab);
        harness.run();
        assert_eq!(
            harness.ctx.memory(|m| m.focused()),
            None,
            "the status bar must not hold a Tab stop"
        );
    }
}
