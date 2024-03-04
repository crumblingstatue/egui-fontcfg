//! Provides a simple dialog for the user to customize the fonts in an egui application
//!
//! ## Basic usage
//!
//! You should keep an [`egui::FontDefinitions`] around in your application state, and edit it
//! with [`FontCfgUi::show`].
//!
//! The ui will automatically apply the changes to the egui context when the user clicks the `Apply`
//! button.
//!
//! This library doesn't handle serialization, but it's fairly easy to do it yourself:
//!
//! - Make sure `egui`'s `serialize` feature is enabled
//! - Serialize the [`egui::FontFamily`] of your font data
//! - Serialize [`CustomFontPaths`], and use [`load_custom_fonts`] to load the custom fonts
//! that the user added.
#![warn(missing_docs)]

use {
    egui::{ahash::HashMap, FontData, FontDefinitions},
    std::collections::BTreeMap,
};

/// The state of the font configuration ui
#[derive(Default)]
pub struct FontCfgUi {
    name_buf: String,
    path_buf: String,
    err_msg: String,
    add_new: bool,
}

/// Keeps track of custom font paths added by the user
///
/// The key is the identifier of the font, the value is the path to the font.
pub type CustomFontPaths = HashMap<String, String>;

/// Helper function to load custom fonts from a [`CustomFontPaths`] to a [`FontData`].
pub fn load_custom_fonts(
    custom: &CustomFontPaths,
    font_data: &mut BTreeMap<String, FontData>,
) -> std::io::Result<()> {
    for (k, v) in custom {
        let data = std::fs::read(v)?;
        font_data.insert(k.to_owned(), FontData::from_owned(data));
    }
    Ok(())
}

/// Message returned by [`FontCfgUi::show`]
pub enum FontDefsUiMsg {
    /// No event happened
    None,
    /// A save was requested
    SaveRequest,
}

impl FontCfgUi {
    /// Show the font definitions ui
    ///
    /// # Arguments
    ///
    /// - `ui`: The [`egui::Ui`] to show the ui on
    /// - `font_defs`: The [`egui::FontDefinitions`] to edit
    /// - `custom`: An optional [`CustomFontPaths`] to save custom font paths to
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        font_defs: &mut FontDefinitions,
        mut custom: Option<&mut CustomFontPaths>,
    ) -> FontDefsUiMsg {
        let mut msg = FontDefsUiMsg::None;
        ui.set_max_width(300.0);
        ui.horizontal(|ui| {
            ui.heading("Fonts");
            if ui.button("+").clicked() {
                self.add_new = true;
                self.err_msg.clear();
            }
        });
        if self.add_new {
            ui.add(
                egui::TextEdit::singleline(&mut self.name_buf).hint_text("Identifier for new font"),
            );
            ui.add(egui::TextEdit::singleline(&mut self.path_buf).hint_text("Path to new font"));
            if ui.button("Add new font").clicked() {
                let font_data = match std::fs::read(&self.path_buf) {
                    Ok(data) => data,
                    Err(e) => {
                        self.err_msg = e.to_string();
                        return FontDefsUiMsg::None;
                    }
                };
                let data = egui::FontData::from_owned(font_data);
                font_defs.font_data.insert(self.name_buf.clone(), data);
                if let Some(custom) = &mut custom {
                    custom.insert(self.name_buf.clone(), self.path_buf.clone());
                }
                self.name_buf.clear();
                self.path_buf.clear();
                self.err_msg.clear();
                self.add_new = false;
            }
        }
        if !self.err_msg.is_empty() {
            ui.label(egui::RichText::new(&self.err_msg).color(egui::Color32::DARK_RED));
        }
        font_defs.font_data.retain(|name, _font| {
            let mut retain = true;
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.button("-").clicked() {
                    if let Some(custom) = &mut custom {
                        custom.remove(name);
                    }
                    retain = false;
                }
            });
            retain
        });
        ui.separator();
        ui.heading("Families");
        let mut push_new_to = None;
        font_defs.families.retain(|family, fonts| {
            let mut retain = true;
            ui.horizontal(|ui| {
                ui.label(family.to_string());
                if ui.button("+").clicked() {
                    push_new_to = Some(family.clone());
                }
                if ui.button("-").clicked() {
                    retain = false;
                }
            });
            fonts.retain_mut(|font_name| {
                let mut retain = true;
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(font_name);
                    if ui.button("-").clicked() {
                        retain = false;
                    }
                });
                retain
            });
            retain
        });
        if let Some(key) = push_new_to {
            font_defs
                .families
                .get_mut(&key)
                .unwrap()
                .push(String::new());
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("✅ Apply").clicked() {
                ui.ctx().set_fonts(font_defs.clone());
            }
            if ui.button("💾 Save").clicked() {
                msg = FontDefsUiMsg::SaveRequest;
            }
        });
        msg
    }
}

/// A convenience window wrapper around [`FontCfgUi`], to show it in a window
#[derive(Default)]
pub struct FontCfgWindow {
    ui: FontCfgUi,
    /// Whether the window should be open
    pub open: bool,
}

impl FontCfgWindow {
    /// Show the font defs ui window
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        font_defs: &mut FontDefinitions,
        custom: Option<&mut CustomFontPaths>,
    ) -> FontDefsUiMsg {
        let mut msg = FontDefsUiMsg::None;
        egui::Window::new("Font definitions")
            .open(&mut self.open)
            .show(ctx, |ui| {
                msg = self.ui.show(ui, font_defs, custom);
            });
        msg
    }
}
