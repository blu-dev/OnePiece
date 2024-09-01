use std::{fmt::Display, hash::Hash, marker::PhantomData};

use egui::{style::WidgetVisuals, AboveOrBelow, ComboBox, Rect, TextWrapMode, Ui, WidgetText};

pub trait EnumValue {
    type Variant: PartialEq + Eq + Display + 'static;

    const VARIANTS: &'static [Self::Variant];

    fn get_variant(&self) -> Self::Variant;
    fn convert(self, new_variant: &Self::Variant) -> Self;
}

pub struct EnumComboBox<E: EnumValue> {
    inner: ComboBox,
    phantom: PhantomData<E>,
}

impl<E: EnumValue> EnumComboBox<E> {
    fn from_combo_box(combo_box: ComboBox) -> Self {
        Self {
            inner: combo_box,
            phantom: PhantomData,
        }
    }

    pub fn new(id_source: impl Hash, label: impl Into<WidgetText>) -> Self {
        Self::from_combo_box(ComboBox::new(id_source, label))
    }

    pub fn from_id_source(id_source: impl Hash) -> Self {
        Self::from_combo_box(ComboBox::from_id_source(id_source))
    }

    pub fn from_label(label: impl Into<WidgetText>) -> Self {
        Self::from_combo_box(ComboBox::from_label(label))
    }

    pub fn height(self, height: f32) -> Self {
        Self::from_combo_box(self.inner.height(height))
    }

    pub fn icon(
        self,
        icon_fn: impl FnOnce(&Ui, Rect, &WidgetVisuals, bool, AboveOrBelow) + 'static,
    ) -> Self {
        Self::from_combo_box(self.inner.icon(icon_fn))
    }

    pub fn truncate(self) -> Self {
        Self::from_combo_box(self.inner.truncate())
    }

    pub fn wrap(self) -> Self {
        Self::from_combo_box(self.inner.wrap())
    }

    pub fn wrap_mode(self, wrap_mode: TextWrapMode) -> Self {
        Self::from_combo_box(self.inner.wrap_mode(wrap_mode))
    }

    pub fn show(self, ui: &mut Ui, value: &mut E) -> egui::Response {
        let current = value.get_variant();
        self.inner
            .selected_text(current.to_string())
            .show_ui(ui, |ui| {
                for variant in E::VARIANTS.iter() {
                    if ui
                        .selectable_label(*variant == current, variant.to_string())
                        .clicked()
                    {
                        // SAFETY: We are doing an zero-clone conversion which must occur this way due to borrowing rules
                        unsafe {
                            let current = std::ptr::read(value);
                            std::ptr::write(value, current.convert(variant));
                        }
                    }
                }
            })
            .response
    }
}
