use std::collections::HashSet;

use egui::{Context, TextEdit, Ui};
use egui_extras::{Column, TableBuilder};

use crate::i18n::UiText;
use crate::scanner::{RegionKind, SearchResult};
use crate::utils::clipboard::copy_to_clipboard;

const PAGE_SIZE: usize = 1000;

#[derive(Default, Clone)]
pub struct ResultsTableState {
    pub filter: String,
    pub selected: HashSet<usize>,
    pub page: usize,
    pub last_filter: String,
}

pub fn results_table_ui(
    ctx: &Context,
    ui: &mut Ui,
    state: &mut ResultsTableState,
    results: &[SearchResult],
    text: &UiText,
) {
    let filtered_indices = filter_indices(results, &state.filter);
    let target_indices = |state: &ResultsTableState| -> Vec<usize> {
        if state.selected.is_empty() {
            return filtered_indices.clone();
        }
        filtered_indices
            .iter()
            .copied()
            .filter(|i| state.selected.contains(i))
            .collect()
    };

    let total_pages = page_count(filtered_indices.len());
    if state.last_filter != state.filter {
        state.page = 0;
        state.last_filter = state.filter.clone();
    }
    if state.page >= total_pages {
        state.page = total_pages.saturating_sub(1);
    }
    let (page_start, page_end) = page_bounds(state.page, filtered_indices.len());
    let page_indices = if filtered_indices.is_empty() {
        &[]
    } else {
        &filtered_indices[page_start..page_end]
    };

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(text.results_filter_label());
            let edit = TextEdit::singleline(&mut state.filter)
                .hint_text(text.results_filter_hint())
                .desired_width(ui.available_width());
            ui.add(edit);
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button(text.button_copy_all()).clicked() {
                let indices = target_indices(state);
                let payload = format_results_subset(results, &indices, text);
                copy_to_clipboard(ctx, payload);
            }
            if ui.button(text.button_save_all()).clicked() {
                let indices = target_indices(state);
                let subset = results_from_indices(results, &indices);
                save_results(&subset, text);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(text.label_page(state.page + 1, total_pages.max(1)));
                if ui
                    .add_enabled(
                        state.page + 1 < total_pages,
                        egui::Button::new(text.button_next_page()),
                    )
                    .clicked()
                {
                    state.page = (state.page + 1).min(total_pages.saturating_sub(1));
                }
                if ui
                    .add_enabled(state.page > 0, egui::Button::new(text.button_prev_page()))
                    .clicked()
                {
                    state.page = state.page.saturating_sub(1);
                }
            });
        });
    });

    if results.is_empty() {
        ui.add_space(8.0);
        ui.label(egui::RichText::new(text.label_no_results_yet()).color(egui::Color32::GRAY));
        return;
    }

    let row_height = 26.0;
    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(150.0))
        .column(Column::remainder().at_least(260.0))
        .column(Column::auto().at_least(120.0))
        .column(Column::auto().at_least(80.0))
        .column(Column::auto().at_least(80.0))
        .header(row_height, |mut header| {
            header.col(|ui| {
                header_with_actions(
                    ctx,
                    ui,
                    text.column_query(),
                    results,
                    state,
                    &filtered_indices,
                    |r| &r.query,
                    text,
                )
            });
            header.col(|ui| {
                header_with_actions(
                    ctx,
                    ui,
                    text.column_match(),
                    results,
                    state,
                    &filtered_indices,
                    |r| &r.matched,
                    text,
                )
            });
            header.col(|ui| {
                ui.label(text.column_address());
                if ui
                    .small_button("📋")
                    .on_hover_text(text.tooltip_copy_addresses())
                    .clicked()
                {
                    let col = collect_column_subset(results, &filtered_indices, |r| {
                        format!("0x{:X}", r.address)
                    });
                    copy_to_clipboard(ctx, col.join("\n"));
                }
            });
            header.col(|ui| {
                ui.label(text.column_region());
            });
            header.col(|ui| {
                ui.label(text.column_encoding());
            });
        })
        .body(|mut body| {
            for &row_idx in page_indices {
                let res = &results[row_idx];
                body.row(row_height, |mut row| {
                    row.col(|ui| selectable_cell(ui, state, row_idx, &res.query, ctx, res, text));
                    row.col(|ui| selectable_cell(ui, state, row_idx, &res.matched, ctx, res, text));
                    row.col(|ui| {
                        ui.label(format!("0x{:X}", res.address));
                    });
                    row.col(|ui| {
                        ui.label(region_label(res.region, text));
                    });
                    row.col(|ui| {
                        ui.label(encoding_label(res.encoding));
                    });
                });
            }
        });
}

fn filter_indices(results: &[SearchResult], filter: &str) -> Vec<usize> {
    let trimmed = filter.trim();
    if trimmed.is_empty() {
        return (0..results.len()).collect();
    }
    let f = trimmed.to_ascii_lowercase();
    results
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.query.to_ascii_lowercase().contains(&f) || r.matched.to_ascii_lowercase().contains(&f)
        })
        .map(|(i, _)| i)
        .collect()
}

fn page_count(total: usize) -> usize {
    if total == 0 {
        1
    } else {
        (total + PAGE_SIZE - 1) / PAGE_SIZE
    }
}

fn page_bounds(page: usize, total: usize) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }
    let start = page.saturating_mul(PAGE_SIZE);
    let end = (start + PAGE_SIZE).min(total);
    (start.min(total), end)
}

fn selectable_cell(
    ui: &mut Ui,
    state: &mut ResultsTableState,
    row_idx: usize,
    text: &str,
    ctx: &Context,
    res: &SearchResult,
    locale: &UiText,
) {
    let resp = ui.selectable_label(state.selected.contains(&row_idx), text);
    if resp.clicked() {
        if !state.selected.insert(row_idx) {
            state.selected.remove(&row_idx);
        }
    }
    resp.context_menu(|ui| {
        if ui.button(locale.context_copy_cell()).clicked() {
            copy_to_clipboard(ctx, text.to_string());
            ui.close();
        }
        if ui.button(locale.context_copy_row()).clicked() {
            copy_to_clipboard(ctx, format_full_row(res, locale));
            ui.close();
        }
    });
}

fn header_with_actions<F>(
    ctx: &Context,
    ui: &mut Ui,
    title: &str,
    results: &[SearchResult],
    state: &ResultsTableState,
    indices: &[usize],
    getter: F,
    text: &UiText,
) where
    F: Fn(&SearchResult) -> &str,
{
    ui.horizontal(|ui| {
        ui.label(title);
        if ui
            .small_button("📋")
            .on_hover_text(text.action_copy_column())
            .clicked()
        {
            let subset = indices_for_action(indices, state);
            let col = collect_column_subset(results, &subset, |r| getter(r).to_string());
            copy_to_clipboard(ctx, col.join("\n"));
        }
        if ui
            .small_button("💾")
            .on_hover_text(text.action_save_column())
            .clicked()
        {
            let subset = indices_for_action(indices, state);
            let col = collect_column_subset(results, &subset, |r| getter(r).to_string());
            let _ = save_lines(col);
        }
    });
}

fn indices_for_action(indices: &[usize], state: &ResultsTableState) -> Vec<usize> {
    if state.selected.is_empty() {
        return indices.to_vec();
    }
    indices
        .iter()
        .copied()
        .filter(|i| state.selected.contains(i))
        .collect()
}

fn collect_column_subset<F>(results: &[SearchResult], indices: &[usize], getter: F) -> Vec<String>
where
    F: Fn(&SearchResult) -> String,
{
    indices
        .iter()
        .filter_map(|&i| results.get(i))
        .map(getter)
        .collect()
}

fn format_results_subset(results: &[SearchResult], indices: &[usize], text: &UiText) -> String {
    results_from_indices(results, indices)
        .iter()
        .map(|r| format_full_row(r, text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn results_from_indices<'a>(
    results: &'a [SearchResult],
    indices: &[usize],
) -> Vec<&'a SearchResult> {
    indices.iter().filter_map(|&i| results.get(i)).collect()
}

fn format_full_row(res: &SearchResult, text: &UiText) -> String {
    format!(
        "{}\t{}\t0x{:X}\t{}\t{}",
        res.query,
        res.matched,
        res.address,
        region_label(res.region, text),
        encoding_label(res.encoding)
    )
}

fn region_label(kind: RegionKind, text: &UiText) -> &'static str {
    match kind {
        RegionKind::Private => text.region_private(),
        RegionKind::Image => text.region_image(),
        RegionKind::Mapped => text.region_mapped(),
        RegionKind::Other => text.region_other(),
    }
}

fn encoding_label(enc: crate::scanner::Encoding) -> &'static str {
    match enc {
        crate::scanner::Encoding::Ascii => "ASCII",
        crate::scanner::Encoding::Utf16 => "UTF-16",
    }
}

fn save_results(results: &[&SearchResult], text: &UiText) {
    let text = results
        .iter()
        .map(|r| format_full_row(r, text))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = save_text(text);
}

fn save_lines(lines: Vec<String>) -> std::io::Result<()> {
    save_text(lines.join("\n"))
}

fn save_text(text: String) -> std::io::Result<()> {
    if let Some(path) = rfd::FileDialog::new()
        .set_file_name("rss-strings-results.txt")
        .save_file()
    {
        std::fs::write(path, text)?;
    }
    Ok(())
}
