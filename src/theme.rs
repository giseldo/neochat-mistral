use crate::storage::ThemeChoice;
use egui::{
    Color32, Context, CornerRadius, FontData, FontFamily, FontId, FontInsert, FontPriority,
    InsertFontFamily, Margin, Stroke, SystemTheme, TextStyle, Theme, ThemePreference, Vec2,
    ViewportCommand,
};
use std::collections::BTreeMap;

// Cores e medidas vêm do design system "Mistral Vibe" (tokens.json).
// Cada valor abaixo cita o token de origem; troque aqui, não nas telas.

/// `bg-state-brand`, tema escuro (orange-600).
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0xDE, 0x44, 0x08);
/// `bg-state-brand`, tema claro (orange-500).
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0xFA, 0x50, 0x0F);
/// Preenchimento destrutivo com texto branco (red-700, 7:1 com branco).
pub const DESTRUCTIVE: Color32 = Color32::from_rgb(0xAD, 0x0E, 0x00);
pub const MAX_CHAT_WIDTH: f32 = 768.0;

pub fn preference(choice: ThemeChoice) -> ThemePreference {
    match choice {
        ThemeChoice::Sistema => ThemePreference::System,
        ThemeChoice::Claro => ThemePreference::Light,
        ThemeChoice::Escuro => ThemePreference::Dark,
    }
}

pub fn apply(ctx: &Context, choice: ThemeChoice) {
    ctx.set_theme(preference(choice));
    ctx.style_mut_of(Theme::Dark, |style| tune(style, true));
    ctx.style_mut_of(Theme::Light, |style| tune(style, false));
}

pub fn sync_title_bar(ctx: &Context, choice: ThemeChoice) {
    let theme = match choice {
        ThemeChoice::Sistema => SystemTheme::SystemDefault,
        ThemeChoice::Claro => SystemTheme::Light,
        ThemeChoice::Escuro => SystemTheme::Dark,
    };
    ctx.send_viewport_cmd(ViewportCommand::SetTheme(theme));
}

pub fn install_fonts(ctx: &Context) {
    for path in [
        "C:\\Windows\\Fonts\\SegUIVar.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
    ] {
        if let Ok(bytes) = std::fs::read(path) {
            ctx.add_font(FontInsert::new(
                "Segoe UI",
                FontData::from_owned(bytes),
                vec![InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Highest,
                }],
            ));
            break;
        }
    }

    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        ctx.add_font(FontInsert::new(
            "Segoe UI Symbol",
            FontData::from_owned(bytes),
            vec![InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            }],
        ));
    }

    if let Ok(bytes) = std::fs::read("C:\\Windows\\Fonts\\CascadiaMono.ttf") {
        ctx.add_font(FontInsert::new(
            "Cascadia Mono",
            FontData::from_owned(bytes),
            vec![InsertFontFamily {
                family: FontFamily::Monospace,
                priority: FontPriority::Highest,
            }],
        ));
    }
}

/// Cor de sucesso para texto pequeno: green-800 no claro (green-700 dá só 3,8:1
/// sobre o creme) e
/// `text-success` (green-500) no escuro.
pub fn success_color(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x44, 0xBA, 0x82)
    } else {
        Color32::from_rgb(0x2B, 0x69, 0x0B)
    }
}

/// Os tokens semânticos do design system, resolvidos para um tema.
struct Palette {
    bg_default: Color32,
    bg_subtle: Color32,
    bg_muted: Color32,
    bg_sidebar: Color32,
    text_default: Color32,
    text_subtle: Color32,
    text_informative: Color32,
    text_destructive: Color32,
    text_warning: Color32,
    brand: Color32,
    selection: Color32,
    border: Color32,
    border_darker: Color32,
    soft: Color32,
    soft_hover: Color32,
    soft_press: Color32,
    ghost_hover: Color32,
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg_default: Color32::from_rgb(0x10, 0x10, 0x13), // steel-950
            bg_subtle: Color32::from_rgb(0x1A, 0x1A, 0x1E),  // steel-900
            bg_muted: Color32::from_rgb(0x27, 0x27, 0x2B),   // steel-800
            bg_sidebar: Color32::from_rgb(0x17, 0x17, 0x1B), // steel-925
            text_default: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            text_subtle: Color32::from_rgb(0xCC, 0xCC, 0xD1), // steel-300
            text_informative: Color32::from_rgb(0x55, 0xB3, 0xFB), // blue-400
            text_destructive: Color32::from_rgb(0xF6, 0x6C, 0x60), // red-400
            text_warning: Color32::from_rgb(0xFF, 0x65, 0x29), // orange-400
            brand: ACCENT_DARK,
            selection: Color32::from_rgb(0x7A, 0x23, 0x00), // orange-900
            border: rgba(255, 255, 255, 26),                // border-default
            border_darker: rgba(255, 255, 255, 38),         // border-darker
            soft: rgba(255, 255, 255, 20),                  // bg-state-soft
            soft_hover: rgba(255, 255, 255, 26),            // bg-state-soft-hover
            soft_press: rgba(255, 255, 255, 38),            // bg-state-soft-press
            ghost_hover: rgba(255, 255, 255, 20),           // bg-state-ghost-hover
        }
    } else {
        Palette {
            bg_default: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            bg_subtle: Color32::from_rgb(0xFB, 0xFB, 0xF8), // cream-50
            bg_muted: Color32::from_rgb(0xF5, 0xF4, 0xEF),  // cream-100
            bg_sidebar: Color32::from_rgb(0xF8, 0xF7, 0xF3), // cream-25
            text_default: Color32::from_rgb(0x20, 0x1F, 0x1C), // cream-950
            text_subtle: Color32::from_rgb(0x61, 0x5F, 0x57), // cream-700
            text_informative: Color32::from_rgb(0x00, 0x82, 0xE6), // blue-600
            text_destructive: Color32::from_rgb(0xE5, 0x13, 0x00), // red-600
            text_warning: Color32::from_rgb(0xDE, 0x44, 0x08), // orange-600
            brand: ACCENT_LIGHT,
            selection: Color32::from_rgb(0xFF, 0xDC, 0xCF), // orange-100
            border: rgba(0, 0, 0, 26),                      // border-default
            border_darker: rgba(0, 0, 0, 38),               // border-darker
            soft: rgba(0, 0, 0, 15),                        // bg-state-soft
            soft_hover: rgba(0, 0, 0, 20),                  // bg-state-soft-hover
            soft_press: rgba(0, 0, 0, 26),                  // bg-state-soft-press
            ghost_hover: rgba(0, 0, 0, 10),                 // bg-state-ghost-hover
        }
    }
}

fn tune(style: &mut egui::Style, dark: bool) {
    style.visuals = visuals(&palette(dark), dark);
    style.spacing.item_spacing = Vec2::new(8.0, 8.0); // space-2
    style.spacing.button_padding = Vec2::new(12.0, 7.0);
    style.spacing.window_margin = Margin::same(16); // space-4
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.interact_size.y = 32.0; // space-8: altura de controles
    style.spacing.combo_width = 170.0;
    style.spacing.combo_height = 420.0;
    style.spacing.scroll.bar_width = 4.0;
    style.spacing.scroll.floating = false;
    style.spacing.scroll.dormant_background_opacity = 0.15;
    style.spacing.scroll.active_background_opacity = 0.3;
    style.spacing.scroll.interact_background_opacity = 0.45;
    style.spacing.scroll.dormant_handle_opacity = 0.55;
    style.spacing.scroll.active_handle_opacity = 0.8;
    style.spacing.scroll.interact_handle_opacity = 1.0;
    style.animation_time = 0.15; // 150ms, como abas e itens de menu
    style.text_styles = BTreeMap::from([
        (
            TextStyle::Heading,
            FontId::new(20.0, FontFamily::Proportional),
        ),
        // `body`: 16/24
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        // `label`: 14/20
        (
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        // `caption`: 12/16
        (
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(13.5, FontFamily::Monospace),
        ),
    ]);
}

fn visuals(p: &Palette, dark: bool) -> egui::Visuals {
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    v.panel_fill = p.bg_subtle;
    v.window_fill = p.bg_default;
    v.extreme_bg_color = p.bg_default;
    v.faint_bg_color = p.bg_muted;
    v.code_bg_color = p.bg_muted;
    v.override_text_color = Some(p.text_default);
    v.weak_text_color = Some(p.text_subtle);
    v.hyperlink_color = p.text_informative;
    v.error_fg_color = p.text_destructive;
    v.warn_fg_color = p.text_warning;
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0, p.text_default);
    v.window_corner_radius = CornerRadius::same(14); // radius-xl
    v.menu_corner_radius = CornerRadius::same(10); // radius-lg
    v.window_stroke = Stroke::new(1.0, p.border);

    widgets(&mut v.widgets, p);
    v.widgets.noninteractive.bg_fill = p.bg_subtle;
    v.widgets.noninteractive.weak_bg_fill = p.bg_subtle;
    v.widgets.noninteractive.bg_stroke = Stroke::NONE;
    v
}

fn widgets(widgets: &mut egui::style::Widgets, p: &Palette) {
    let text = p.text_default;
    let apply = |w: &mut egui::style::WidgetVisuals, bg: Color32, stroke: Stroke| {
        w.bg_fill = bg;
        w.weak_bg_fill = bg;
        w.bg_stroke = stroke;
        w.fg_stroke = Stroke::new(1.0, text);
        w.corner_radius = CornerRadius::same(8); // radius-md
    };

    // Botão "suave": bg-state-soft, com hover e pressionado um degrau acima.
    apply(&mut widgets.inactive, p.soft, Stroke::NONE);
    apply(
        &mut widgets.hovered,
        p.soft_hover,
        Stroke::new(1.0, p.border_darker),
    );
    apply(
        &mut widgets.active,
        p.soft_press,
        Stroke::new(1.0, p.border_darker),
    );
    apply(
        &mut widgets.open,
        p.soft_hover,
        Stroke::new(1.0, p.border_darker),
    );
    widgets.noninteractive.corner_radius = CornerRadius::same(8);
}

#[derive(Debug, Clone)]
pub struct ChatColors {
    pub background: Color32,
    pub sidebar_bg: Color32,
    pub surface: Color32,
    pub border: Color32,
    pub user_bubble: Color32,
    pub user_text: Color32,
    pub assistant_bubble: Color32,
    pub assistant_text: Color32,
    pub assistant_border: Color32,
    pub muted: Color32,
    pub accent: Color32,
    /// `bg-state-ghost-hover`: fundo de linhas e botões de ícone em hover.
    pub row_hover: Color32,
    /// `bg-state-soft`: item de menu ativo.
    pub row_selected: Color32,
}

pub fn chat_colors(theme: Theme) -> ChatColors {
    let p = palette(theme == Theme::Dark);
    ChatColors {
        background: p.bg_subtle,
        sidebar_bg: p.bg_sidebar,
        surface: p.bg_default,
        border: p.border,
        user_bubble: p.ghost_hover,
        user_text: p.text_default,
        // A resposta do assistente não tem balão no design system.
        assistant_bubble: Color32::TRANSPARENT,
        assistant_text: p.text_default,
        assistant_border: Color32::TRANSPARENT,
        // `text-subtle`: legível (6:1 ou mais); `text-muted` não passa de 3,2:1 no claro.
        muted: p.text_subtle,
        accent: p.brand,
        row_hover: p.ghost_hover,
        row_selected: p.soft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luminance(c: Color32) -> f32 {
        let channel = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }

    fn contrast(a: Color32, b: Color32) -> f32 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    #[test]
    fn theme_choice_maps_to_preference() {
        assert!(matches!(
            preference(ThemeChoice::Sistema),
            ThemePreference::System
        ));
        assert!(matches!(
            preference(ThemeChoice::Claro),
            ThemePreference::Light
        ));
        assert!(matches!(
            preference(ThemeChoice::Escuro),
            ThemePreference::Dark
        ));
    }

    #[test]
    fn body_text_is_legible_in_both_themes() {
        for dark in [false, true] {
            let p = palette(dark);
            for ground in [p.bg_default, p.bg_subtle, p.bg_sidebar] {
                assert!(contrast(p.text_default, ground) >= 7.0, "text_default dark={dark}");
                assert!(contrast(p.text_subtle, ground) >= 4.5, "text_subtle dark={dark}");
                assert!(
                    contrast(success_color(dark), ground) >= 4.0,
                    "success dark={dark}"
                );
            }
        }
    }

    #[test]
    fn destructive_fill_reads_with_white_text() {
        assert!(contrast(DESTRUCTIVE, Color32::WHITE) >= 4.5);
    }
}
