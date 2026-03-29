//! Foreground window / app context for capture and policy.

#[derive(Debug, Clone)]
pub(crate) struct WindowBounds {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct AppContext {
    pub(crate) app_name: Option<String>,
    pub(crate) window_title: Option<String>,
    pub(crate) bounds: Option<WindowBounds>,
}

impl AppContext {
    pub(crate) fn same_as(&self, other: &AppContext) -> bool {
        self.app_name == other.app_name
            && self.window_title == other.window_title
            && self.bounds.as_ref().map(|b| (b.x, b.y, b.width, b.height))
                == other.bounds.as_ref().map(|b| (b.x, b.y, b.width, b.height))
    }

    pub(crate) fn as_compound_text(&self) -> String {
        format!(
            "{} {}",
            self.app_name.clone().unwrap_or_default(),
            self.window_title.clone().unwrap_or_default()
        )
        .to_lowercase()
    }
}
