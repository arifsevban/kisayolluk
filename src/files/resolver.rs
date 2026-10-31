use crate::files::file_context::FileContext;
use crate::platform::windows::explorer;

#[derive(Debug, Clone)]
pub enum FileResolverResult {
    Selected(FileContext),
    NoSelection,
    MultipleSelection(usize),
    UnsupportedApp,
    Error(String),
}

pub fn resolve_selected_file() -> FileResolverResult {
    explorer::get_explorer_selection()
}

