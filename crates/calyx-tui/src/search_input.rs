//! Keys while the search bar is being edited ('/'): typing, Backspace, Enter
//! (search), Esc (clears the text; on an empty bar, leaves) and Ctrl+U
//! (clears, as in a shell).

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, PartialEq)]
pub enum SearchEdit {
    /// Still editing.
    Typing,
    /// Enter: run the search.
    Submit,
    /// Esc on an empty bar: stop editing.
    Leave,
}

/// Applies `key` to the search text `input`.
pub fn edit(input: &mut String, key: KeyEvent) -> SearchEdit {
    // AltGr arrives as Ctrl+Alt on Windows ('@', '#', '['...): still text.
    let ctrl_only =
        key.modifiers.contains(KeyModifiers::CONTROL) && !key.modifiers.contains(KeyModifiers::ALT);
    match key.code {
        KeyCode::Enter => SearchEdit::Submit,
        KeyCode::Esc if input.is_empty() => SearchEdit::Leave,
        KeyCode::Esc => {
            input.clear();
            SearchEdit::Typing
        }
        KeyCode::Char('u') if ctrl_only => {
            input.clear();
            SearchEdit::Typing
        }
        KeyCode::Backspace => {
            input.pop();
            SearchEdit::Typing
        }
        KeyCode::Char(c) if !ctrl_only => {
            input.push(c);
            SearchEdit::Typing
        }
        _ => SearchEdit::Typing,
    }
}

#[cfg(test)]
#[path = "search_input_tests.rs"]
mod tests;
