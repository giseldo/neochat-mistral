use super::{i18n::tr, NeoChat};
use crate::skills;
use egui::{Align2, RichText, ScrollArea, TextEdit, Window};

/// Janela para buscar e ativar skills na conversa atual.
pub(super) fn skills_window(app: &mut NeoChat, ctx: &egui::Context) {
    if !app.skills_open {
        return;
    }
    let active: Vec<String> = app
        .store
        .conversations
        .active()
        .map(|c| c.skills.clone())
        .unwrap_or_default();
    let mut open = true;
    let mut toggle: Option<String> = None;
    let mut filter = std::mem::take(&mut app.skills_filter);

    Window::new(tr("Skills"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([460.0, 480.0])
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(RichText::new(tr("Instruções de texto que o modelo aplica nesta conversa (máx. 4).")).weak());
            ui.add_space(6.0);
            ui.add(
                TextEdit::singleline(&mut filter)
                    .hint_text(tr("Buscar skills"))
                    .desired_width(f32::INFINITY),
            );
            ui.add_space(6.0);
            let query = filter.trim().to_lowercase();
            let mut shown = 0;
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for category in skills::categories() {
                    let items: Vec<&skills::Skill> = skills::catalog()
                        .iter()
                        .filter(|s| s.category == category && s.matches(&query))
                        .collect();
                    if items.is_empty() {
                        continue;
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(category).strong().small());
                    for skill in items {
                        shown += 1;
                        let mut on = active.contains(&skill.id);
                        let before = on;
                        ui.checkbox(&mut on, &skill.title).on_hover_text(&skill.description);
                        if on != before {
                            toggle = Some(skill.id.clone());
                        }
                    }
                }
                if shown == 0 {
                    ui.label(RichText::new(tr("Nenhuma skill encontrada")).weak());
                }
            });
        });

    app.skills_filter = filter;
    if let Some(id) = toggle {
        app.toggle_skill(&id);
    }
    if !open {
        app.skills_open = false;
    }
}
