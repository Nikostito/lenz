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

#[test]
fn mac_shortcut_labels_read_cmd_and_opt_where_the_desktop_runs_on_macos() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = session(dir.path());
    assert_eq!(s.key_hint(CommandId::FormatDocument).as_deref(), Some("Ctrl+Alt+L"));
    s.set_mac_shortcuts(true);
    assert_eq!(s.key_hint(CommandId::FormatDocument).as_deref(), Some("Cmd+Opt+L"), "IntelliJ's Reformat Code on macOS");
    assert_eq!(s.key_hint(CommandId::NextProjectTab).as_deref(), Some("Cmd+Opt+]"));
    assert_eq!(s.primary_modifier(), "Cmd");
    assert!(s.welcome_tip().contains("Cmd+Opt+] / Cmd+Opt+["), "{}", s.welcome_tip());
    // The same press runs it: Command is the primary modifier, so ⌘⌥L arrives as the Ctrl+Alt+L binding.
    assert_eq!(s.key_command(&crate::keymap::KeyPress::new("l", true, false, true).unwrap()), Some(CommandId::FormatDocument));
    assert_eq!(crate::keymap::mac_spelling("Ctrl+Shift+F12"), "Cmd+Shift+F12");
    assert_eq!(crate::keymap::mac_spelling("F5"), "F5");
    assert_eq!(crate::keymap::mac_spelling("Ctrl++"), "Cmd++");
}

#[test]
fn recent_projects_carry_the_branch_read_off_the_ui_thread() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
    let mut s = session(dir.path());
    s.dispatch(Command::OpenRoot(repo.clone()));
    settle(&mut s, scanned);
    settle(&mut s, |s| s.recent_projects("").first().is_some_and(|r| r.branch.is_some()));
    let found = s.recent_projects("");
    assert_eq!((found[0].name.as_str(), found[0].branch.as_deref()), ("repo", Some("main")));
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/feature/login\n").unwrap();
    s.dispatch(Command::RefreshRecentBranches);
    settle(&mut s, |s| s.recent_projects("login").len() == 1);
}
