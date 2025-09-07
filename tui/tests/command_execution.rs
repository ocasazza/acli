use atui::{
    app::App,
    command::{CommandInputMode, CtagOperation},
    event_handler::EventHandler,
    screens::Screen,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use std::env;

#[test]
fn test_command_execution_and_scrolling() {
    // Setup: Create a new app
    env::set_var("ATLASSIAN_URL", "https://example.com");
    env::set_var("ATLASSIAN_USERNAME", "user");
    env::set_var("ATLASSIAN_API_TOKEN", "token");
    let mut app = App::new().expect("Failed to create app");

    // 1. Navigate to CommandExecution screen
    app.switch_screen(Screen::CommandExecution);

    // 2. Select a command (e.g., ctag list)
    app.command_input.set_command(atui::command::AvailableCommand::Ctag {
        operation: CtagOperation::List,
        description: "List labels".to_string(),
    });
    app.command_input.mode = CommandInputMode::TypingArgs;

    // 3. Simulate executing the command by pressing Enter
    let enter_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    EventHandler::handle_event(&mut app, enter_event).expect("Failed to handle Enter event");

    // 4. Assert that command output is populated
    // The mock executor should return a known string.
    // Since we are not mocking, we expect the command to fail because we are not in a project context.
    // This is fine for testing the UI logic.
    assert!(!app.command_output.is_empty());
    assert!(app
        .command_output
        .join("\n")
        .contains("No valid context for command execution"));

    // 5. Simulate scrolling down
    let arrow_down_event = Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    app.command_output = vec!["line1".to_string(), "line2".to_string(), "line3".to_string()];
    app.command_output_scroll = 0;

    EventHandler::handle_event(&mut app, arrow_down_event.clone())
        .expect("Failed to handle Arrow Down event");
    assert_eq!(app.command_output_scroll, 1);

    EventHandler::handle_event(&mut app, arrow_down_event.clone())
        .expect("Failed to handle Arrow Down event");
    assert_eq!(app.command_output_scroll, 2);

    // Should not scroll past the end
    EventHandler::handle_event(&mut app, arrow_down_event.clone())
        .expect("Failed to handle Arrow Down event");
    assert_eq!(app.command_output_scroll, 2);

    // 6. Simulate scrolling up
    let arrow_up_event = Event::Key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
    EventHandler::handle_event(&mut app, arrow_up_event.clone())
        .expect("Failed to handle Arrow Up event");
    assert_eq!(app.command_output_scroll, 1);

    EventHandler::handle_event(&mut app, arrow_up_event.clone())
        .expect("Failed to handle Arrow Up event");
    assert_eq!(app.command_output_scroll, 0);

    // Should not scroll past the beginning
    EventHandler::handle_event(&mut app, arrow_up_event.clone())
        .expect("Failed to handle Arrow Up event");
    assert_eq!(app.command_output_scroll, 0);
}
