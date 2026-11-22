use super::{i18n::tr, NeoChat};
use crate::theme;
use egui::{CollapsingHeader, Frame, Label, Margin, Panel, RichText, ScrollArea, Ui};

/// Painel à direita com o que foi enviado ao modelo na última requisição.
pub(super) fn inspector(app: &mut NeoChat, ui: &mut Ui) {
    if !app.inspector_open {
        return;
    }
    let colors = theme::chat_colors(ui.ctx().theme());
    let last_usage = app
        .store
        .conversations
        .active()
        .and_then(|c| c.messages.iter().rev().find(|m| !m.is_user()))
        .map(|m| (m.prompt_tokens, m.completion_tokens, m.cached_tokens, m.total_tokens));
    let busy = app.busy();
    let mut copy: Option<String> = None;
    let mut view_skill: Option<String> = None;

    Panel::right("inspector")
        .resizable(true)
        .default_size(360.0)
        .min_size(260.0)
        .max_size(640.0)
        .frame(Frame::new().fill(colors.sidebar_bg).inner_margin(Margin::symmetric(12, 12)))
        .show(ui, |ui| {
            ui.label(RichText::new(tr("Inspetor")).size(16.0).color(colors.assistant_text));
            ui.add_space(8.0);
            let Some(req) = app.last_request.as_ref() else {
                ui.label(RichText::new(tr("Nenhuma requisição enviada ainda.")).weak());
                return;
            };
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                CollapsingHeader::new(tr("Resumo")).default_open(true).show(ui, |ui| {
                    let line = |ui: &mut Ui, k: &str, v: String| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new(format!("{k}:")).color(colors.muted));
                            ui.add(Label::new(v).selectable(true));
                        });
                    };
                    line(ui, "model", req.model.clone());
                    line(ui, "reasoning_effort", req.reasoning_effort.unwrap_or("—").to_string());
                    line(ui, "temperature", req.temperature.map_or("—".into(), |t| format!("{t:.2}")));
                    line(ui, "max_tokens", req.max_tokens.map_or("auto".into(), |t| t.to_string()));
                    line(ui, "prompt_cache_key", req.cache_key.clone().unwrap_or_else(|| "—".into()));
                    let chars: usize = req.messages.iter().map(|m| m.1).sum();
                    line(ui, "mensagens", req.messages.len().to_string());
                    line(ui, "tamanho", format!("{chars} caracteres (≈ {} tokens)", chars / 4));
                    if busy {
                        line(ui, "status", "aguardando resposta…".into());
                    }
                });

                CollapsingHeader::new(format!("{} ({})", tr("Skills enviadas"), req.skills.len()))
                    .default_open(true)
                    .show(ui, |ui| {
                        if req.skills.is_empty() {
                            ui.label(RichText::new(tr("nenhuma")).weak());
                        }
                        for id in &req.skills {
                            let size = crate::skills::find(id).map_or(0, |s| s.content.chars().count());
                            ui.horizontal(|ui| {
                                ui.label(format!("✦ {id} — {size} caracteres"));
                                if ui.small_button(tr("ver")).clicked() {
                                    view_skill = Some(id.clone());
                                }
                            });
                        }
                    });

                CollapsingHeader::new(format!("{} ({})", tr("Sugestões de skills"), app.suggestions.len()))
                    .show(ui, |ui| {
                        if app.suggestions.is_empty() {
                            ui.label(RichText::new(tr("nenhuma")).weak());
                        }
                        for s in &app.suggestions {
                            ui.label(format!("✦ {} — {:.1} ({})", s.title, s.score, s.matched.join(", ")));
                        }
                    });

                CollapsingHeader::new(tr("Prompt de sistema enviado")).show(ui, |ui| {
                    ui.add(Label::new(RichText::new(&req.system_prompt).size(12.0)).selectable(true).wrap());
                });

                CollapsingHeader::new(tr("Mensagens enviadas")).show(ui, |ui| {
                    for (i, (role, chars, images)) in req.messages.iter().enumerate() {
                        let extra = if *images > 0 { format!(" + {images} imagem(ns)") } else { String::new() };
                        ui.label(format!("{i}. {role} — {chars} caracteres{extra}"));
                    }
                });

                CollapsingHeader::new(tr("JSON da requisição")).show(ui, |ui| {
                    if ui.button(tr("Copiar JSON")).clicked() {
                        copy = Some(req.json.clone());
                    }
                    ui.add(Label::new(RichText::new(&req.json).monospace().size(11.0)).selectable(true).wrap());
                });

                if let Some((prompt, completion, cached, total)) = last_usage {
                    CollapsingHeader::new(tr("Última resposta")).default_open(true).show(ui, |ui| {
                        let show = |v: Option<u32>| v.map_or("—".to_string(), |n| n.to_string());
                        ui.label(format!("prompt_tokens: {}", show(prompt)));
                        ui.label(format!("completion_tokens: {}", show(completion)));
                        ui.label(format!("cached_tokens: {}", show(cached)));
                        ui.label(format!("total_tokens: {}", show(total)));
                    });
                }
            });
        });

    if let Some(text) = copy {
        ui.ctx().copy_text(text);
    }
    if view_skill.is_some() {
        app.skill_detail = view_skill;
    }
}
