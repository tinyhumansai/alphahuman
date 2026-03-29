#[derive(Debug, Clone)]
struct WindowBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone)]
struct AppContext {
    app_name: Option<String>,
    window_title: Option<String>,
    bounds: Option<WindowBounds>,
}

impl AppContext {
    fn same_as(&self, other: &AppContext) -> bool {
        self.app_name == other.app_name
            && self.window_title == other.window_title
            && self.bounds.as_ref().map(|b| (b.x, b.y, b.width, b.height))
                == other.bounds.as_ref().map(|b| (b.x, b.y, b.width, b.height))
    }

    fn as_compound_text(&self) -> String {
        format!(
            "{} {}",
            self.app_name.clone().unwrap_or_default(),
            self.window_title.clone().unwrap_or_default()
        )
        .to_lowercase()
    }
}
