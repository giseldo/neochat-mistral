use super::NeoChat;
use crate::storage::{Language, ThemeChoice, DEFAULT_TEMPERATURE};
use egui::{Align2, Button, ComboBox, RichText, Slider, TextEdit, Window};

pub(super) fn settings_window(app: &mut NeoChat, ctx: &egui::Context) {
    if !app.settings_open {
        return;
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.settings_open = false;
        return;
    }

    let mut open = true;
    let mut save = false;
    let mut test = false;
    let mut fetch_models = false;
    let mut close = false;
    let mut login = false;
    let mut logout = false;
    let pending = app.test_pending;
    let status = app.test_status.clone();
    let models_pending = app.models_pending;
    let models_status = app.models_status.clone();
    let login_pending = app.login_pending;
    let login_status = app.login_status.clone();
    let models = app.model_list();
    let is_logged_in = !app.settings_key.trim().is_empty() || !app.api_key.trim().is_empty();

    let tab_id = egui::Id::unique("settings_tab");
    let mut tab = ctx.data_mut(|data| data.get_temp::<usize>(tab_id).unwrap_or(0));

    let screen = ctx.content_rect();
    let max_height = (screen.height() - 32.0).max(120.0);
    let max_width = (screen.width() - 32.0).max(280.0);

    Window::new(super::i18n::tr("Configurações"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(530.0)
        .default_height(max_height.min(720.0))
        .max_width(max_width)
        .max_height(max_height)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (index, label) in [super::i18n::tr("Conta"), super::i18n::tr("Modelo"), super::i18n::tr("Geração"), super::i18n::tr("Aparência"), super::i18n::tr("Sobre"), super::i18n::tr("Conversas")].iter().enumerate() {
                    if ui.selectable_label(tab == index, *label).clicked() {
                        tab = index;
                    }
                }
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);
            egui::ScrollArea::vertical()
                .id_salt("settings_content")
                .max_height((max_height.min(720.0) - 180.0).max(40.0))
                .min_scrolled_height((max_height.min(560.0) - 180.0).max(40.0))
                .show(ui, |ui| {
                    if tab == 0 {
                    ui.label(RichText::new(super::i18n::tr("Conta Mistral")).heading());
                    ui.label(RichText::new(super::i18n::tr("Gerencie sua conexão e chave de acesso.")).small().color(ui.visuals().weak_text_color()));
                    if is_logged_in {
                        ui.label(RichText::new(super::i18n::tr("● Conectado à Mistral")).color(ui.visuals().hyperlink_color));
                        if ui.add_enabled(!login_pending, Button::new(super::i18n::tr("Sair da conta"))).on_hover_text(super::i18n::tr("Desconectar e remover a chave salva")).clicked() { logout = true; }
                    } else {
                        let label = if login_pending { super::i18n::tr("Aguardando login no navegador...") } else { super::i18n::tr("Entrar com a conta Mistral") };
                        if ui.add_enabled(!login_pending, Button::new(label)).clicked() { login = true; }
                    }
                    if let Some((ok, message)) = &login_status {
                        let color = if *ok { ui.visuals().hyperlink_color } else { ui.visuals().error_fg_color };
                        ui.label(RichText::new(message).small().color(color));
                    }
                    ui.add_space(6.0);
                    ui.label(RichText::new(super::i18n::tr("Chave da API")).strong());
                    ui.add(
                        TextEdit::singleline(&mut app.settings_key)
                            .password(true)
                            .desired_width(f32::INFINITY)
                            .hint_text(super::i18n::tr("sua chave da API")),
                    );
                    ui.add_space(2.0);
                    ui.hyperlink_to(
                        super::i18n::tr("Obter uma chave no console da Mistral"),
                        "https://console.mistral.ai/api-keys",
                    );
                    ui.add_space(10.0);
                    ui.label(RichText::new(super::i18n::tr("A chave é salva criptografada com o Windows (DPAPI).")).small().color(ui.visuals().weak_text_color()));
                    let test_label = if pending { super::i18n::tr("Testando...") } else { super::i18n::tr("Testar conexão") };
                    if ui.add_enabled(!pending, Button::new(test_label)).clicked() { test = true; }
                    if let Some((ok, message)) = &status {
                        let color = if *ok { ui.visuals().hyperlink_color } else { ui.visuals().error_fg_color };
                        ui.label(RichText::new(message).small().color(color));
                    }
                    }
                    if tab == 1 {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(super::i18n::tr("Modelo")).heading());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let fetch_label = if models_pending {
                                super::i18n::tr("Carregando...")
                            } else {
                                super::i18n::tr("Carregar todos os modelos")
                            };
                            if ui
                                .add_enabled(
                                    !models_pending,
                                    Button::new(RichText::new(fetch_label).small()),
                                )
                                .on_hover_text(
                                    super::i18n::tr("Buscar a lista completa de modelos disponíveis na API da Mistral"),
                                )
                                .clicked()
                            {
                                fetch_models = true;
                            }
                        });
                    });
                    let combo_width = (ui.available_width() - 8.0).clamp(200.0, 455.0);
                    ComboBox::from_id_salt("settings_model")
                        .width(combo_width)
                        .height(360.0)
                        .selected_text(app.settings_model.clone())
                        .show_ui(ui, |ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.spacing_mut().interact_size.y = 24.0;
                            for model in &models {
                                ui.selectable_value(&mut app.settings_model, model.clone(), model);
                            }
                        });
                    ui.label(RichText::new(super::i18n::tr("Identificador do modelo selecionado na API")).small().color(ui.visuals().weak_text_color()));
                    if let Some((ok, message)) = &models_status {
                        let color = if *ok {
                            crate::theme::success_color(ui.visuals().dark_mode)
                        } else {
                            ui.visuals().error_fg_color
                        };
                        ui.label(RichText::new(message).small().color(color));
                    }

                    }
                    if tab == 2 {
                    let has_custom_params = (app.settings_temperature - DEFAULT_TEMPERATURE).abs() > 0.005
                        || !app.settings_max_tokens.trim().is_empty()
                        || !app.settings_prompt_caching;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(super::i18n::tr("Geração")).heading());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    has_custom_params,
                                    Button::new(RichText::new(super::i18n::tr("Restaurar padrões")).small()),
                                )
                                .on_hover_text(super::i18n::tr("Voltar para temperatura 0.70, limite automático e cache ativado"))
                                .clicked()
                            {
                                app.settings_temperature = DEFAULT_TEMPERATURE;
                                app.settings_max_tokens.clear();
                                app.settings_prompt_caching = true;
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(super::i18n::tr("Temperatura:"));
                        ui.add(
                            Slider::new(&mut app.settings_temperature, 0.0..=1.5)
                                .step_by(0.05)
                                .fixed_decimals(2),
                        );
                    });
                    ui.label(
                        RichText::new(
                            super::i18n::tr("0.00 = determinístico/código · 0.70 = equilibrado · >1.00 = mais criativo"),
                        )
                        .small()
                        .color(ui.visuals().weak_text_color()),
                    );

                    ui.add_space(6.0);
                    ui.label(RichText::new(super::i18n::tr("Máximo de tokens")).strong());
                    ui.horizontal_wrapped(|ui| {
                        for (label, value) in [(super::i18n::tr("Automático"), ""), ("512", "512"), ("2048", "2048"), ("4096", "4096")] {
                            if ui.selectable_label(app.settings_max_tokens == value, label).clicked() {
                                app.settings_max_tokens = value.to_string();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(super::i18n::tr("Personalizado:"));
                        ui.add(TextEdit::singleline(&mut app.settings_max_tokens).desired_width(110.0).hint_text(super::i18n::tr("Automático")));
                    });
                    app.settings_max_tokens.retain(|c| c.is_ascii_digit());

                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut app.settings_prompt_caching,
                        super::i18n::tr("Prompt Caching (reduz custos e latência)"),
                    )
                    .on_hover_text(
                        super::i18n::tr("Envia chave de sessão à Mistral para reaproveitar contexto e obter até 90% de desconto nos tokens de entrada em cache."),
                    );
                    ui.label(
                        RichText::new(
                            super::i18n::tr("Desconto de 90% em tokens já processados no histórico da conversa (blocos a partir de 64 tokens)."),
                        )
                        .small()
                        .color(ui.visuals().weak_text_color()),
                    );

                    }
                    if tab == 3 {
                    ui.label(RichText::new(super::i18n::tr("Aparência")).heading());
                    ui.label(RichText::new(super::i18n::tr("Escolha como o NeoChat aparece no seu computador.")).small().color(ui.visuals().weak_text_color()));
                    ui.label(RichText::new(super::i18n::tr("Idioma")).strong());
                    ComboBox::from_id_salt("settings_language")
                        .selected_text(app.settings_language.label())
                        .show_ui(ui, |ui| {
                            for language in [Language::Portugues, Language::Ingles, Language::Frances] {
                                ui.selectable_value(&mut app.settings_language, language, language.label());
                            }
                        });
                    ui.add_space(8.0);
                    ui.label(RichText::new(super::i18n::tr("Tema")).strong());
                    ui.horizontal(|ui| {
                        for choice in [
                            ThemeChoice::Sistema,
                            ThemeChoice::Claro,
                            ThemeChoice::Escuro,
                        ] {
                            if ui.selectable_label(app.settings_theme == choice, super::i18n::tr(choice.label())).clicked() {
                                app.settings_theme = choice;
                            }
                        }
                    });

                    }
                    if tab == 5 {
                        ui.label(RichText::new(super::i18n::tr("Conversas")).heading());
                        ui.label(super::i18n::tr("Exclua todo o histórico de conversas deste dispositivo."));
                        ui.add_space(10.0);
                        if ui.button(super::i18n::tr("Excluir todas as conversas")).clicked() {
                            app.delete_all_confirm = true;
                        }
                    }
                    if tab == 4 {
                    ui.label(RichText::new(super::i18n::tr("Sobre o NeoChat")).heading());
                    ui.label(format!("Versão {}", env!("CARGO_PKG_VERSION")));
                    ui.label(super::i18n::tr("Desenvolvido por Giseldo"));
                    ui.label(super::i18n::tr("giseldo@gmail.com"));
                    ui.label(super::i18n::tr("Licença MIT"));
                    ui.hyperlink_to(
                        super::i18n::tr("Repositório do projeto"),
                        "https://github.com/giseldo/neochat-mistral",
                    );
                    ui.label(
                        RichText::new(format!("Dados em {}", crate::storage::data_dir().display()))
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .add(Button::new(RichText::new(super::i18n::tr("Salvar")).strong()))
                    .clicked()
                {
                    save = true;
                }
                if ui.button(super::i18n::tr("Cancelar")).clicked() {
                    close = true;
                }
            });
        });
    ctx.data_mut(|data| data.insert_temp(tab_id, tab));

    app.settings_open = open && !close;
    if test {
        app.start_test(ctx);
    }
    if fetch_models {
        app.start_fetch_models(ctx);
    }
    if save {
        app.apply_settings();
    }
    if login {
        app.start_login(ctx);
    }
    if logout {
        app.logout();
    }
}


pub(super) fn confirm_delete_all(app: &mut NeoChat, ctx: &egui::Context) {
    if !app.delete_all_confirm { return; }
    let mut open = true;
    let mut confirm = false;
    let mut cancel = false;
    Window::new(super::i18n::tr("Excluir todas as conversas"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(super::i18n::tr("Excluir todas as conversas e suas mensagens deste dispositivo?"));
            ui.label(RichText::new(super::i18n::tr("Essa ação não pode ser desfeita.")).weak());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(super::i18n::tr("Excluir todas")).clicked() { confirm = true; }
                if ui.button(super::i18n::tr("Cancelar")).clicked() { cancel = true; }
            });
        });
    if confirm {
        app.delete_all_conversations();
        app.settings_open = false;
    }
    if confirm || cancel || !open { app.delete_all_confirm = false; }
}
