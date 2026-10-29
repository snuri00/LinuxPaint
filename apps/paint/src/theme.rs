use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeChoice {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub window: Color32,
    pub titlebar: Color32,
    pub ribbon: Color32,
    pub ribbon_border: Color32,
    pub workspace: Color32,
    pub canvas_border: Color32,
    pub card: Color32,
    pub card_stroke: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub selected: Color32,
    pub selected_border: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub control: Color32,
    pub control_stroke: Color32,
    pub statusbar: Color32,
    pub ruler_bg: Color32,
    pub ruler_fg: Color32,
    pub flyout: Color32,
}

const fn a(r: u8, g: u8, b: u8, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(
        ((r as u16 * alpha as u16) / 255) as u8,
        ((g as u16 * alpha as u16) / 255) as u8,
        ((b as u16 * alpha as u16) / 255) as u8,
        alpha,
    )
}

fn tint(c: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha)
}

pub fn palette(dark: bool, accent: Color32) -> Palette {
    if dark {
        Palette {
            dark: true,
            window: Color32::from_rgb(0x24, 0x24, 0x24),
            titlebar: Color32::from_rgb(0x30, 0x30, 0x30),
            ribbon: Color32::from_rgb(0x2E, 0x2E, 0x2E),
            ribbon_border: a(0, 0, 0, 0x40),
            workspace: Color32::from_rgb(0x1C, 0x1C, 0x1C),
            canvas_border: a(255, 255, 255, 0x17),
            card: Color32::from_rgb(0x38, 0x38, 0x38),
            card_stroke: a(0, 0, 0, 0x5C),
            divider: a(255, 255, 255, 0x1A),
            text: Color32::WHITE,
            text_secondary: a(255, 255, 255, 0x8C),
            text_disabled: a(255, 255, 255, 0x4D),
            icon: Color32::from_rgb(0xF0, 0xF0, 0xF0),
            hover: a(255, 255, 255, 0x12),
            pressed: a(255, 255, 255, 0x29),
            selected: tint(accent, 0x45),
            selected_border: accent,
            accent,
            accent_text: Color32::WHITE,
            control: a(255, 255, 255, 0x1A),
            control_stroke: a(255, 255, 255, 0x26),
            statusbar: Color32::from_rgb(0x24, 0x24, 0x24),
            ruler_bg: Color32::from_rgb(0x2E, 0x2E, 0x2E),
            ruler_fg: Color32::from_rgb(0xA0, 0xA0, 0xA0),
            flyout: Color32::from_rgb(0x38, 0x38, 0x38),
        }
    } else {
        Palette {
            dark: false,
            window: Color32::from_rgb(0xFA, 0xFA, 0xFA),
            titlebar: Color32::from_rgb(0xEB, 0xEB, 0xEB),
            ribbon: Color32::WHITE,
            ribbon_border: a(0, 0, 0, 0x14),
            workspace: Color32::from_rgb(0xDE, 0xDD, 0xDA),
            canvas_border: a(0, 0, 0, 0x29),
            card: Color32::WHITE,
            card_stroke: a(0, 0, 0, 0x14),
            divider: a(0, 0, 0, 0x1F),
            text: a(0, 0, 0, 0xCC),
            text_secondary: a(0, 0, 0, 0x8C),
            text_disabled: a(0, 0, 0, 0x4D),
            icon: Color32::from_rgb(0x2E, 0x34, 0x36),
            hover: a(0, 0, 0, 0x12),
            pressed: a(0, 0, 0, 0x29),
            selected: tint(accent, 0x30),
            selected_border: accent,
            accent,
            accent_text: Color32::WHITE,
            control: a(0, 0, 0, 0x14),
            control_stroke: a(0, 0, 0, 0x26),
            statusbar: Color32::from_rgb(0xFA, 0xFA, 0xFA),
            ruler_bg: Color32::WHITE,
            ruler_fg: Color32::from_rgb(0x70, 0x70, 0x70),
            flyout: Color32::WHITE,
        }
    }
}

pub fn apply(ctx: &egui::Context, p: &Palette) {
    let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = p.window;
    v.window_fill = p.flyout;
    v.extreme_bg_color = p.control;
    v.faint_bg_color = p.ribbon;
    v.override_text_color = Some(p.text);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(12);
    v.window_stroke = Stroke::new(1.0, p.card_stroke);
    v.selection.bg_fill = p.accent;
    v.selection.stroke = Stroke::new(1.0, p.accent_text);
    v.hyperlink_color = p.accent;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(6);
        w.fg_stroke = Stroke::new(1.0, p.text);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.divider);
    v.widgets.inactive.weak_bg_fill = p.control;
    v.widgets.inactive.bg_fill = p.control;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.control_stroke);
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.control_stroke);
    v.widgets.active.weak_bg_fill = p.pressed;
    v.widgets.active.bg_fill = p.accent;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent);
    v.slider_trailing_fill = true;
    ctx.set_theme(if p.dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 4.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.interact_size.y = 28.0;
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
        s.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    });
}
