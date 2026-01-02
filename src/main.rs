mod common;
mod ws_2d;
mod ws_3d;

mod app;
mod ui;

use ui::ui;
use app::{App, CurrentScreen, PopupFocus, MenuItem, PuzzleType, PuzzleSize};
use color_eyre::Result;
use ratatui::{crossterm::{event::{self, Event, KeyCode, MouseEventKind, EnableMouseCapture, DisableMouseCapture}, execute}, DefaultTerminal};
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    execute!(io::stdout(), EnableMouseCapture)?;
    let mut app = App::new();
    let result = run(terminal, &mut app);
    execute!(io::stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    match app.current_screen {
                        CurrentScreen::Menu => {
                            match key.code {
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => app.nav_up(),
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => app.nav_down(),
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    match app.menu_items[app.selected_item] {
                                        MenuItem::NewPuzzle => {
                                            app.current_screen = CurrentScreen::NewPuzzlePopup;
                                            app.popup_focus = PopupFocus::TypeSelector;
                                        }
                                        MenuItem::LoadPuzzle => app.open_load_popup(),
                                        MenuItem::Quit => app.quit(),
                                    }
                                }
                                _ => {}
                            }
                        },
                        CurrentScreen::NewPuzzlePopup => {
                            match key.code {
                                // Close popup
                                KeyCode::Esc => app.current_screen = CurrentScreen::Menu,
                                
                                // Navigate vertical (Focus switching)
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                                    app.popup_focus = match app.popup_focus {
                                        PopupFocus::TypeSelector => PopupFocus::SizeSelector,
                                        PopupFocus::SizeSelector => PopupFocus::Buttons,
                                        PopupFocus::Buttons => PopupFocus::Buttons,
                                    };
                                }
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                                    app.popup_focus = match app.popup_focus {
                                        PopupFocus::TypeSelector => PopupFocus::TypeSelector,
                                        PopupFocus::SizeSelector => PopupFocus::TypeSelector,
                                        PopupFocus::Buttons => PopupFocus::SizeSelector,
                                    };
                                }
                                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                                    match app.popup_focus {
                                        PopupFocus::TypeSelector => {
                                            app.puzzle_type = match app.puzzle_type {
                                                PuzzleType::TwoD => PuzzleType::ThreeD,
                                                PuzzleType::ThreeD => PuzzleType::TwoD,
                                            };
                                        }
                                        PopupFocus::SizeSelector => {
                                            app.puzzle_size = match app.puzzle_size {
                                                PuzzleSize::Small => PuzzleSize::Large,
                                                PuzzleSize::Medium => PuzzleSize::Small,
                                                PuzzleSize::Large => PuzzleSize::Medium,
                                            };
                                        }
                                        PopupFocus::Buttons => if app.popup_button_index == 0 { app.popup_button_index = 1; } else { app.popup_button_index = 0; },
                                    }
                                }
                                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                                    match app.popup_focus {
                                        PopupFocus::TypeSelector => {
                                            app.puzzle_type = match app.puzzle_type {
                                                PuzzleType::TwoD => PuzzleType::ThreeD,
                                                PuzzleType::ThreeD => PuzzleType::TwoD,
                                            };
                                        }
                                        PopupFocus::SizeSelector => {
                                            app.puzzle_size = match app.puzzle_size {
                                                PuzzleSize::Small => PuzzleSize::Medium,
                                                PuzzleSize::Medium => PuzzleSize::Large,
                                                PuzzleSize::Large => PuzzleSize::Small,
                                            };
                                        }
                                        PopupFocus::Buttons => if app.popup_button_index == 0 { app.popup_button_index = 1; } else { app.popup_button_index = 0; },
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    match app.popup_focus {
                                        PopupFocus::Buttons => if app.popup_button_index == 0 { app.start_game(); } else { app.current_screen = CurrentScreen::Menu; }
                                        _ => {} 
                                    }
                                }
                                _ => {}
                            }
                        },
                        CurrentScreen::LoadPuzzlePopup => {
                            match key.code {
                                KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.nav_load_down(),
                                KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.nav_load_up(),
                                KeyCode::Enter => app.load_selected_puzzle(),
                                KeyCode::Esc => app.close_load_popup(),
                                KeyCode::Char('d') | KeyCode::Char('D') => app.open_delete_popup(),
                                _ => {}
                            }
                        }
                        CurrentScreen::DeleteSavePopup => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => app.delete_selected_save(),
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.close_delete_popup(),
                                _ => {} 
                            }
                        }
                        CurrentScreen::Game => {
                            match key.code {
                                // Grid Navigation
                                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => app.move_cursor_up(),
                                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => app.move_cursor_down(),
                                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => app.move_cursor_left(),
                                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => app.move_cursor_right(),
                                // Word List Scrolling
                                KeyCode::Char('j') | KeyCode::Char('J') => app.scroll_list_down(),
                                KeyCode::Char('k') | KeyCode::Char('K') => app.scroll_list_up(),
                                // Visiblility toggle
                                KeyCode::Char('v') | KeyCode::Char('V') => app.toggle_found_visibility(),
                                // Selection
                                KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selection(),
                                // Save
                                KeyCode::Char('g') | KeyCode::Char('G') => app.open_save_popup(),
                                // Exit
                                KeyCode::Esc => app.trigger_exit_warning(),
                                _ => {}
                            }
                        }
                        CurrentScreen::SavePuzzlePopup => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => app.save_puzzle(),
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.close_save_popup(),
                                _ => {}
                            }
                        }
                        CurrentScreen::ExitWarning => {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_exit(),
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_exit(),
                                _ => {}
                            }
                        }
                        CurrentScreen::Win => {
                            match key.code {
                                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Esc => app.current_screen = CurrentScreen::Menu,
                                _ => {}
                            }
                        },
                    }
                }
                Event::Mouse(mouse) => {
                    match app.current_screen {
                        CurrentScreen::Game =>
                        match mouse.kind {
                            MouseEventKind::ScrollDown => app.scroll_list_down(),
                            MouseEventKind::ScrollUp => app.scroll_list_up(),
                            _ => {}
                        },
                        CurrentScreen::LoadPuzzlePopup => 
                        match mouse.kind {
                            MouseEventKind::ScrollDown => app.nav_load_down(),
                            MouseEventKind::ScrollUp => app.nav_load_up(),
                            _ => {}
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if app.exit {
            return Ok(());
        }
    }
}