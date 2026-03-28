use egui::{ComboBox, RichText, TextEdit, Ui};

use crate::i18n::UiText;
use crate::process::{ProcessInfo, ProcessStatus};

#[derive(Default, Clone)]
pub struct ProcessPickerState {
    pub filter: String,
    pub selected_pid: Option<u32>,
}

pub fn process_picker_ui(
    ui: &mut Ui,
    state: &mut ProcessPickerState,
    processes: &[ProcessInfo],
    text: &UiText,
) -> Option<u32> {
    let filter = state.filter.trim().to_ascii_lowercase();
    let filtered: Vec<&ProcessInfo> = processes
        .iter()
        .filter(|p| {
            if filter.is_empty() {
                return true;
            }
            if let Ok(pid) = filter.parse::<u32>() {
                return p.pid == pid;
            }
            p.name.to_ascii_lowercase().contains(&filter)
                || p.path
                    .as_ref()
                    .map(|s| s.to_ascii_lowercase().contains(&filter))
                    .unwrap_or(false)
        })
        .collect();

    let mut new_selection = None;

    ui.horizontal(|ui| {
        ui.label(text.process_label());
        ComboBox::from_id_salt("process_combo")
            .selected_text(selected_label(state.selected_pid, processes, text))
            .show_ui(ui, |ui| {
                for proc in filtered {
                    let mut label_text = format!("{} (PID {})", proc.name, proc.pid);
                    label_text.push_str(if proc.is_64_bit {
                        " • x64"
                    } else {
                        " • x86"
                    });
                    if proc.status != ProcessStatus::Ok {
                        label_text.push_str(text.process_limited_suffix());
                    }

                    let text = if proc.status == ProcessStatus::Ok {
                        RichText::new(label_text)
                    } else {
                        RichText::new(label_text).color(egui::Color32::GRAY)
                    };

                    let resp = ui.selectable_value(&mut state.selected_pid, Some(proc.pid), text);
                    if resp.clicked() {
                        new_selection = Some(proc.pid);
                    }
                }
            });

        ui.add(
            TextEdit::singleline(&mut state.filter).hint_text(text.process_filter_hint()),
        );
    });

    new_selection
}

fn selected_label(pid: Option<u32>, processes: &[ProcessInfo], text: &UiText) -> String {
    if let Some(pid) = pid {
        if let Some(p) = processes.iter().find(|p| p.pid == pid) {
            return format!("{} (PID {})", p.name, p.pid);
        }
        return format!("PID {}", pid);
    }
    text.process_not_selected().to_string()
}
