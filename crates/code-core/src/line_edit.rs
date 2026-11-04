//! Caret editing in a one-line text box (Search Everywhere's query, Search in Project's fields): both adapters keep the
//! text and a byte caret and map their keys to [`Edit`]s, so the box moves and deletes the same way in the window and
//! the terminal.
use unicode_segmentation::UnicodeSegmentation;

/// One caret step or deletion. Word steps stop as the editor's word motions do: spaces go with the word beside them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit {
    Left,
    Right,
    WordLeft,
    WordRight,
    Start,
    End,
    Backspace,
    Delete,
    BackspaceWord,
    DeleteWord,
}

/// `caret` kept within `text` on a character boundary.
pub fn clamp(text: &str, caret: usize) -> usize {
    text.floor_char_boundary(caret.min(text.len()))
}

/// Apply `edit` at `caret`; whether `text` changed.
pub fn apply(text: &mut String, caret: &mut usize, edit: Edit) -> bool {
    let at = clamp(text, *caret);
    let (moved, removed) = match edit {
        Edit::Left => (previous(text, at), None),
        Edit::Right => (next(text, at), None),
        Edit::WordLeft => (word_stop(text, at, true), None),
        Edit::WordRight => (word_stop(text, at, false), None),
        Edit::Start => (0, None),
        Edit::End => (text.len(), None),
        Edit::Backspace => (previous(text, at), Some(previous(text, at)..at)),
        Edit::Delete => (at, Some(at..next(text, at))),
        Edit::BackspaceWord => (word_stop(text, at, true), Some(word_stop(text, at, true)..at)),
        Edit::DeleteWord => (at, Some(at..word_stop(text, at, false))),
    };
    *caret = moved;
    match removed.filter(|range| !range.is_empty()) {
        Some(range) => { text.replace_range(range, ""); true }
        None => false,
    }
}

/// [`apply`] to a field whose whole text may be selected (Ctrl/Cmd+A): then a step leaves the selection at its start
/// (left, a word left, the start) or its end, and a deletion clears the field. Whether `text` changed.
pub fn apply_selected(text: &mut String, caret: &mut usize, selected: &mut bool, edit: Edit) -> bool {
    if !std::mem::take(selected) { return apply(text, caret, edit); }
    match edit {
        Edit::Left | Edit::WordLeft | Edit::Start => { *caret = 0; false }
        Edit::Right | Edit::WordRight | Edit::End => { *caret = text.len(); false }
        Edit::Backspace | Edit::Delete | Edit::BackspaceWord | Edit::DeleteWord => {
            let changed = !text.is_empty();
            text.clear();
            *caret = 0;
            changed
        }
    }
}

/// Type `typed` at `caret` (control characters dropped), leaving the caret after it.
pub fn insert(text: &mut String, caret: &mut usize, typed: &str) {
    let at = clamp(text, *caret);
    let typed: String = typed.chars().filter(|c| !c.is_control()).collect();
    text.insert_str(at, &typed);
    *caret = at + typed.len();
}

fn previous(text: &str, at: usize) -> usize {
    text.get(..at).unwrap_or_default().grapheme_indices(true).next_back().map_or(0, |(i, _)| i)
}

fn next(text: &str, at: usize) -> usize {
    text.get(at..).unwrap_or_default().graphemes(true).next().map_or(at, |g| at + g.len())
}

/// Spaces, then a run of word characters (letters, digits, `_`) or of other symbols.
fn word_stop(text: &str, at: usize, backward: bool) -> usize {
    let class = |c: char| if c.is_alphanumeric() || c == '_' { 1 } else { 2 };
    if backward {
        let mut chars = text.get(..at).unwrap_or_default().char_indices().rev().skip_while(|(_, c)| c.is_whitespace()).peekable();
        let Some(&(_, first)) = chars.peek() else { return 0 };
        chars.take_while(|(_, c)| !c.is_whitespace() && class(*c) == class(first)).last().map_or(at, |(i, _)| i)
    } else {
        let mut chars = text.get(at..).unwrap_or_default().char_indices().skip_while(|(_, c)| c.is_whitespace()).peekable();
        let Some(&(_, first)) = chars.peek() else { return text.len() };
        chars.find(|(_, c)| c.is_whitespace() || class(*c) != class(first)).map_or(text.len(), |(i, _)| at + i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, caret: usize, edits: &[Edit]) -> (String, usize) {
        let (mut text, mut caret) = (text.to_owned(), caret);
        for edit in edits { apply(&mut text, &mut caret, *edit); }
        (text, caret)
    }

    #[test]
    fn word_steps_stop_at_the_value_the_colon_and_the_key() {
        let text = "column: timestamp";
        let stops: Vec<_> = (1..=4).map(|n| run(text, text.len(), &vec![Edit::WordLeft; n]).1).collect();
        assert_eq!(stops, [8, 6, 0, 0]);
        let stops: Vec<_> = (1..=4).map(|n| run(text, 0, &vec![Edit::WordRight; n]).1).collect();
        assert_eq!(stops, [6, 7, 17, 17]);
    }

    #[test]
    fn typing_and_deleting_happen_at_the_caret() {
        let (mut text, mut caret) = ("sketch.yaml".to_owned(), 6);
        insert(&mut text, &mut caret, " pad\n");
        assert_eq!((text.as_str(), caret), ("sketch pad.yaml", 10));
        assert_eq!(run(&text, caret, &[Edit::BackspaceWord]), ("sketch .yaml".into(), 7));
        assert_eq!(run(&text, caret, &[Edit::BackspaceWord, Edit::BackspaceWord]), (".yaml".into(), 0), "the space goes with the word");
        assert_eq!(run("a_b c", 3, &[Edit::BackspaceWord]), (" c".into(), 0), "`_` is a word character");
        assert_eq!(run(&text, caret, &[Edit::DeleteWord]), ("sketch padyaml".into(), 10));
        assert_eq!(run(&text, caret, &[Edit::Start, Edit::Delete, Edit::End, Edit::Backspace]), ("ketch pad.yam".into(), 13));
        assert_eq!(run(&text, caret, &[Edit::Left, Edit::Left, Edit::Right]), (text.clone(), 9));
        assert_eq!(run("", 5, &[Edit::Backspace, Edit::Delete, Edit::BackspaceWord, Edit::Left]), (String::new(), 0));
    }

    #[test]
    fn a_selected_field_collapses_on_a_step_and_clears_on_a_deletion() {
        let step = |edit| { let (mut text, mut caret, mut selected) = ("draft notes".to_owned(), 3, true); let changed = apply_selected(&mut text, &mut caret, &mut selected, edit); (text, caret, selected, changed) };
        assert_eq!(step(Edit::Left), ("draft notes".into(), 0, false, false));
        assert_eq!(step(Edit::WordRight), ("draft notes".into(), 11, false, false));
        assert_eq!(step(Edit::BackspaceWord), (String::new(), 0, false, true));
        let (mut text, mut caret, mut selected) = ("draft notes".to_owned(), 5, false);
        assert!(apply_selected(&mut text, &mut caret, &mut selected, Edit::BackspaceWord));
        assert_eq!((text.as_str(), caret), (" notes", 0), "nothing selected: as apply");
    }

    #[test]
    fn steps_cross_whole_characters() {
        assert_eq!(run("aé👩🏽‍💻b", 1, &[Edit::Right]), ("aé👩🏽‍💻b".into(), 3));
        assert_eq!(run("aé👩🏽‍💻b", 18, &[Edit::Left]), ("aé👩🏽‍💻b".into(), 3), "an emoji sequence is one step");
        assert_eq!(run("aé👩🏽‍💻b", 18, &[Edit::Backspace]), ("aéb".into(), 3));
        assert_eq!(clamp("é", 1), 0);
    }
}
