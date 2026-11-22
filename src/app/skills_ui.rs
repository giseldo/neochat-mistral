use super::{i18n::tr, NeoChat};
use crate::skills;
use egui::{Align2, RichText, ScrollArea, TextEdit, Window};
use egui_commonmark::CommonMarkViewer;

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
    let mut view: Option<String> = None;
    let mut filter = std::mem::take(&mut app.skills_filter);

    Window::new(tr("Skills"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([460.0, 480.0])
        .anchor(Align2::LEFT_CENTER, [24.0, 0.0])
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
                        ui.horizontal(|ui| {
                            let mut on = active.contains(&skill.id);
                            let before = on;
                            ui.checkbox(&mut on, &skill.title).on_hover_text(&skill.description);
                            if on != before {
                                toggle = Some(skill.id.clone());
                            }
                            if ui.small_button("👁").on_hover_text(tr("Ver skill completa")).clicked() {
                                view = Some(skill.id.clone());
                            }
                        });
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
    if let Some(id) = view {
        app.skill_detail = Some(id);
    }
    if !open {
        app.skills_open = false;
    }
}

/// Janela com o texto completo de uma skill (aberta pela lista, pelos chips ou pelo Inspetor).
pub(super) fn skill_detail_window(app: &mut NeoChat, ctx: &egui::Context) {
    let Some(id) = app.skill_detail.clone() else {
        return;
    };
    let Some(skill) = skills::find(&id) else {
        app.skill_detail = None;
        return;
    };
    let is_active = app
        .store
        .conversations
        .active()
        .is_some_and(|c| c.skills.contains(&id));
    let mut open = true;
    let mut toggle = false;
    let chars = skill.content.chars().count();

    Window::new(&skill.title)
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([620.0, 560.0])
        .anchor(Align2::RIGHT_CENTER, [-24.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&skill.category).strong());
                ui.label(RichText::new(format!("· {chars} {} (≈ {} tokens)", tr("caracteres"), chars / 4)).weak());
                if !skill.tags.is_empty() {
                    ui.label(RichText::new(format!("· {}", skill.tags.join(", "))).weak());
                }
            });
            ui.add_space(4.0);
            ui.label(&skill.description);
            ui.add_space(6.0);
            let label = if is_active { tr("Desativar nesta conversa") } else { tr("Ativar nesta conversa") };
            if ui.button(label).clicked() {
                toggle = true;
            }
            ui.separator();
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if !skill.activation.use_when.is_empty() || !skill.activation.avoid_when.is_empty() {
                    egui::CollapsingHeader::new(tr("Quando usar e quando evitar")).show(ui, |ui| {
                        for text in &skill.activation.use_when {
                            ui.label(format!("✔ {text}"));
                        }
                        for text in &skill.activation.avoid_when {
                            ui.label(format!("✖ {text}"));
                        }
                    });
                    ui.add_space(6.0);
                }
                CommonMarkViewer::new().show(ui, &mut app.skill_cache, &skill.content);
            });
        });

    if toggle {
        app.toggle_skill(&id);
    }
    if !open {
        app.skill_detail = None;
    }
}
