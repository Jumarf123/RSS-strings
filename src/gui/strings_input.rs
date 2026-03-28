use egui::{TextEdit, TextStyle, Ui};

use crate::i18n::UiText;
use crate::utils::text::{MAX_STRINGS, parse_input_lines, sample_input};

#[derive(Clone)]
pub struct StringsInputState {
    pub value: String,
    pub limit: usize,
    pub last_count: usize,
}

impl Default for StringsInputState {
    fn default() -> Self {
        Self {
            value: String::new(),
            limit: MAX_STRINGS,
            last_count: 0,
        }
    }
}

pub fn strings_input_ui(ui: &mut Ui, state: &mut StringsInputState, text: &UiText) -> usize {
    ui.label(text.strings_input_label());
    ui.horizontal(|ui| {
        if ui.button(text.button_clear()).clicked() {
            state.value.clear();
        }
        if ui.button(text.button_insert_sample()).clicked() {
            state.value = sample_input();
        }
        if ui.button(text.button_load_file()).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(text.file_filter_text(), &["txt", "log"])
                .pick_file()
            {
                if let Ok(content) = std::fs::read_to_string(path) {
                    state.value = content;
                }
            }
        }
    });

    egui::ScrollArea::vertical()
        .max_height(ui.available_height() * 0.6)
        .show(ui, |ui| {
            let edit = TextEdit::multiline(&mut state.value)
                .font(TextStyle::Monospace)
                .desired_rows(18)
                .desired_width(f32::INFINITY)
                .hint_text(text.strings_hint(state.limit));
            ui.add(edit);
        });

    let parsed = parse_input_lines(&state.value, state.limit);
    state.last_count = parsed.len();
    let trimmed = state.last_count >= state.limit;

    ui.horizontal(|ui| {
        ui.label(text.label_lines_count(state.last_count, state.limit));
        if trimmed {
            ui.colored_label(egui::Color32::LIGHT_YELLOW, text.label_trimmed(state.limit));
        }
    });

    state.last_count
}
