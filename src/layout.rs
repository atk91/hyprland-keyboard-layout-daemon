#[derive(Clone)]
pub struct Layout {
    layout: String,
}

impl Layout {
    pub fn new(layout: String) -> Layout {
        Layout { layout }
    }

    pub fn code(&self) -> Option<&'static str> {
        match &*self.layout {
            "English (US)" => Some("0"),
            "Russian" => Some("1"),
            _ => None,
        }
    }

    pub fn get_layout(&self) -> &String {
        &self.layout
    }
}
