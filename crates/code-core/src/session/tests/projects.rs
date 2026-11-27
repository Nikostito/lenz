use super::*;
use crate::actions::{CatalogAction, InterfaceAction};
use crate::projects::{Action, Info, Layout};

#[test]
fn project_tab_commands_resolve_centrally_from_the_hosts_projects() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = session(dir.path());
    assert_eq!(s.projects(), Info { count: 1, position: 0, layout: Layout::Tabs });
    assert_eq!(s.action(CommandId::NewProjectTab), CatalogAction::Interface(InterfaceAction::Project(Action::New)));
    assert_eq!(s.action(CommandId::OpenFolderInProjectTab), CatalogAction::Interface(InterfaceAction::Project(Action::OpenFolder)));
    assert_eq!(s.action(CommandId::CloseProjectTab), CatalogAction::Interface(InterfaceAction::Project(Action::Close)));
    assert_eq!(s.action(CommandId::NextProjectTab), CatalogAction::Disabled("Only one project is open"));
    assert_eq!(s.action(CommandId::ProjectsSideBySide), CatalogAction::Disabled("Open a second project first"));
    s.set_projects(Info { count: 3, position: 1, layout: Layout::Tabs });
    assert_eq!(s.action(CommandId::NextProjectTab), CatalogAction::Interface(InterfaceAction::Project(Action::Next)));
    assert_eq!(s.action(CommandId::PreviousProjectTab), CatalogAction::Interface(InterfaceAction::Project(Action::Previous)));
    let toggle = |s: &Session| s.actions().into_iter().find(|a| a.id == CommandId::ProjectsSideBySide).unwrap();
    assert_eq!(toggle(&s).on, Some(false));
    s.set_projects(Info { count: 3, position: 1, layout: Layout::SideBySide });
    assert_eq!(toggle(&s).on, Some(true));
    assert_eq!(CommandId::NextProjectTab.group(), crate::actions::CommandGroup::Projects);
    assert_eq!(crate::actions::CommandGroup::Projects.parent(), Some(crate::actions::CommandGroup::File));
}

#[test]
fn project_titles_name_the_folder_then_the_file_then_the_start_screen_and_mark_unsaved_work() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = session(dir.path());
    assert_eq!(s.project_title(), "Start");
    let root = dir.path().join("alpha");
    std::fs::create_dir(&root).unwrap();
    let path = write(&root, "a.txt", b"saved");
    s.dispatch(Command::OpenRoot(root));
    settle(&mut s, scanned);
    assert_eq!(s.project_title(), "alpha");
    open(&mut s, Command::OpenRetained(path), "a.txt");
    s.enable_text_input_for_test();
    assert!(!s.project_dirty());
    let tab = s.state().active().unwrap().id();
    let expected = s.state().current_source().unwrap().key();
    s.dispatch(Command::Edit { tab, expected, edits: vec![crate::text::TextEdit { range: 0..0, text: "x".into() }], kind: crate::text::EditKind::Replace });
    assert!(s.project_dirty());
}
