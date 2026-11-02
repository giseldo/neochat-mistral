use super::NeoChat;
use crate::theme::{self, MAX_CHAT_WIDTH};
use egui::{
    Align, CornerRadius, Frame, Id, Label, Layout, Margin, Panel, RichText, Sense, Stroke,
    StrokeKind, TextEdit, Ui,
};

pub(super) fn composer(app: &mut NeoChat, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    let colors = theme::chat_colors(ctx.theme());
    let composer_id = Id::unique("composer_input_edit");
    let has_focus = ctx.memory(|m| m.has_focus(composer_id));

    Panel::bottom("composer")
        .resizable(false)
        .frame(Frame::new().fill(colors.background).inner_margin(Margin {
            left: 0,
            right: 0,
            top: 6,
            bottom: 20,
        }))
        .show(ui, |ui| {
            let avail_w = ui.available_width();
            let col_w = (avail_w - 48.0).clamp(260.0, MAX_CHAT_WIDTH);
            let side_pad = ((avail_w - col_w) * 0.5).max(24.0);

            ui.horizontal(|ui| {
                ui.add_space(side_pad);
                ui.vertical(|ui| {
                    ui.set_width(col_w);
                    ui.set_max_width(col_w);

                    if let Some(error) = app.error.clone() {
                        Frame::new()
                            .fill(ui.visuals().error_fg_color.gamma_multiply(0.10))
                            .stroke(Stroke::new(
                                1.0,
                                ui.visuals().error_fg_color.gamma_multiply(0.35),
                            ))
                            .corner_radius(10)
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        Label::new(
                                            RichText::new(error)
                                                .small()
                                                .color(ui.visuals().error_fg_color),
                                        )
                                        .wrap(),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.small_button("×").clicked() {
                                            app.error = None;
                                        }
                                    });
                                });
                            });
                        ui.add_space(8.0);
                    } else if let Some(status) = app.status.clone() {
                        ui.horizontal(|ui| {
                            ui.add(Label::new(
                                RichText::new(status).small().color(colors.muted),
                            ));
                            if ui.small_button("×").clicked() {
                                app.status = None;
                            }
                        });
                        ui.add_space(6.0);
                    }

                    let border_color = if has_focus {
                        colors.muted.gamma_multiply(0.55)
                    } else {
                        colors.border
                    };
                    let border_stroke = Stroke::new(1.0, border_color);

                    let line_count = app.input.lines().count().clamp(1, 6);

                    let card = Frame::new()
                        .fill(colors.surface)
                        .corner_radius(CornerRadius::same(14))
                        .inner_margin(Margin::symmetric(16, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let btn_size = 32.0;
                                let spacing = ui.spacing().item_spacing.x;
                                let (plus, plus_response) = ui.allocate_exact_size(egui::vec2(20.0, 32.0), Sense::hover());
                                let center = plus.center();
                                let stroke = Stroke::new(1.4, colors.muted);
                                ui.painter().line_segment([center + egui::vec2(-6.0, 0.0), center + egui::vec2(6.0, 0.0)], stroke);
                                ui.painter().line_segment([center + egui::vec2(0.0, -6.0), center + egui::vec2(0.0, 6.0)], stroke);
                                plus_response.on_hover_text("Anexos indispon\u{00ed}veis");
                                let show_model = ui.available_width() >= 300.0;
                                let show_think = show_model
                                    && ui.available_width() >= 400.0
                                    && crate::mistral::supports_reasoning(&app.store.config.model);
                                let model_width = if show_model { 130.0 + spacing } else { 0.0 }
                                    + if show_think { 96.0 + spacing } else { 0.0 };
                                let edit_width = (ui.available_width() - btn_size - spacing - model_width).max(60.0);

                                let edit = ui.add_sized(
                                    [edit_width, (line_count as f32 * 21.0).max(btn_size)],
                                    TextEdit::multiline(&mut app.input)
                                        .id(composer_id)
                                        .frame(Frame::NONE)
                                        .align(egui::Align2::LEFT_CENTER)
                                        .hint_text(super::i18n::tr("Digite uma mensagem..."))
                                        .desired_rows(line_count)
                                        .desired_width(f32::INFINITY),
                                );

                                if app.focus_composer {
                                    edit.request_focus();
                                    app.focus_composer = false;
                                }

                                let enter = edit.has_focus()
                                    && ui.input(|i| {
                                        i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                                    });
                                if enter {
                                    while app.input.ends_with('\n') {
                                        app.input.pop();
                                    }
                                }

                                if show_model {
                                    let models = app.model_list();
                                    let mut selected = app.store.config.model.clone();
                                    let display = crate::storage::truncate_chars(&selected, 14);
                                    let mut fetch = false;
                                    egui::ComboBox::from_id_salt("composer_model")
                                        .width(112.0)
                                        .height(300.0)
                                        .selected_text(RichText::new(display).size(13.0).color(colors.muted))
                                        .show_ui(ui, |ui| {
                                            ui.set_min_width(240.0);
                                            for model in &models {
                                                ui.selectable_value(&mut selected, model.clone(), model);
                                            }
                                            ui.separator();
                                            let label = if app.models_pending { super::i18n::tr("Carregando modelos...") } else { super::i18n::tr("Carregar todos os modelos") };
                                            if ui.add_enabled(!app.models_pending, egui::Button::new(label)).clicked() {
                                                fetch = true;
                                                ui.close();
                                            }
                                        });
                                    if selected != app.store.config.model {
                                        app.store.config.model = selected.clone();
                                        app.settings_model = selected;
                                        app.store.save();
                                    }
                                    if fetch {
                                        app.start_fetch_models(&ctx);
                                    }
                                }

                                if show_think {
                                    let mut think = app.store.config.think;
                                    let label = if think { super::i18n::tr("Refletir") } else { super::i18n::tr("Rápido") };
                                    egui::ComboBox::from_id_salt("composer_think")
                                        .width(78.0)
                                        .selected_text(RichText::new(label).size(13.0).color(colors.muted))
                                        .show_ui(ui, |ui| {
                                            ui.set_min_width(200.0);
                                            ui.selectable_value(&mut think, false, super::i18n::tr("Rápido"))
                                                .on_hover_text(super::i18n::tr("Obtenha respostas rápidas"));
                                            ui.selectable_value(&mut think, true, super::i18n::tr("Refletir"))
                                                .on_hover_text(super::i18n::tr("Lide com tarefas complexas"));
                                        });
                                    if think != app.store.config.think {
                                        app.store.config.think = think;
                                        app.store.save();
                                    }
                                }

                                let busy = app.busy();
                                let has_text = !app.input.trim().is_empty();
                                let enabled = busy || has_text;

                                let sense = if enabled {
                                    Sense::click()
                                } else {
                                    Sense::hover()
                                };
                                let (btn_rect, btn_resp) =
                                    ui.allocate_exact_size(egui::vec2(btn_size, btn_size), sense);

                                let (bg_fill, fg_color) = if busy {
                                    (colors.assistant_text, colors.surface)
                                } else if has_text {
                                    let bg = if btn_resp.hovered() {
                                        colors.assistant_text.gamma_multiply(0.85)
                                    } else {
                                        colors.assistant_text
                                    };
                                    (bg, colors.surface)
                                } else {
                                    (
                                        colors.border.gamma_multiply(0.55),
                                        colors.muted.gamma_multiply(0.65),
                                    )
                                };

                                ui.painter()
                                    .rect_filled(btn_rect, CornerRadius::same(8), bg_fill);

                                let c = btn_rect.center();
                                if busy {
                                    let sq = egui::Rect::from_center_size(c, egui::vec2(8.5, 8.5));
                                    ui.painter()
                                        .rect_filled(sq, CornerRadius::same(2), fg_color);
                                } else {
                                    // Seta em pixels, como no botão da referência.
                                    for (x, y, width, height) in [
                                        (-1.0, -7.0, 2.0, 14.0),
                                        (-3.0, -5.0, 2.0, 2.0),
                                        (1.0, -5.0, 2.0, 2.0),
                                        (-5.0, -3.0, 2.0, 2.0),
                                        (3.0, -3.0, 2.0, 2.0),
                                    ] {
                                        let pixel = egui::Rect::from_min_size(
                                            egui::pos2(c.x + x, c.y + y),
                                            egui::vec2(width, height),
                                        );
                                        ui.painter().rect_filled(pixel, CornerRadius::ZERO, fg_color);
                                    }
                                }

                                let tooltip = if busy {
                                    super::i18n::tr("Interromper resposta (Esc)")
                                } else {
                                    super::i18n::tr("Enviar mensagem (Enter)")
                                };
                                if enabled && btn_resp.on_hover_text(tooltip).clicked() {
                                    if busy {
                                        app.stop();
                                    } else {
                                        app.send(&ctx);
                                        edit.request_focus();
                                    }
                                } else if enter && !busy && has_text {
                                    app.send(&ctx);
                                    edit.request_focus();
                                }
                            });
                        });

                    ui.painter().rect_stroke(
                        card.response.rect,
                        CornerRadius::same(14),
                        border_stroke,
                        StrokeKind::Inside,
                    );
                });
            });
        });
}
