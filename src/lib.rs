use {
    egui::{ahash::HashMap, FontData, FontDefinitions},
    std::collections::BTreeMap,
};

#[derive(Default)]
pub struct FontDefsUi {
    name_buf: String,
    path_buf: String,
    err_msg: String,
    add_new: bool,
}

/// Keeps track of custom fonts added by the user
///
/// It's name-path pairs of custom fonts
pub type CustomFonts = HashMap<String, String>;

pub fn load_custom_fonts(
    custom: &CustomFonts,
    font_data: &mut BTreeMap<String, FontData>,
) -> std::io::Result<()> {
    for (k, v) in custom {
        let data = std::fs::read(v)?;
        font_data.insert(k.to_owned(), FontData::from_owned(data));
    }
    Ok(())
}

pub enum FontDefsUiMsg {
    None,
    SaveRequest,
}

impl FontDefsUi {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        font_defs: &mut FontDefinitions,
        mut custom: Option<&mut CustomFonts>,
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

#[derive(Default)]
pub struct FontDefsWindow {
    ui: FontDefsUi,
    pub open: bool,
}

impl FontDefsWindow {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        font_defs: &mut FontDefinitions,
        custom: Option<&mut CustomFonts>,
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
