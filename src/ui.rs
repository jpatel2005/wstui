use std::time::Duration;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line,Span},
    widgets::{{Table, Row, Cell, List, ListItem, Paragraph, Block, Borders, Clear, BorderType, canvas::{Canvas, Line as CanvasLine, Context}}},
    Frame,
    symbols::Marker,
};
use tui_big_text::{BigText, PixelSize};
use crate::{app::{ActivePuzzle, App, CurrentScreen, MenuItem, PopupFocus, PuzzleSize, PuzzleType}, ws_3d};
use crate::ws_3d::grid::Face;

pub fn ui(frame: &mut Frame, app: &mut App) {
    let vertical_layout = Layout::vertical([
        Constraint::Min(0), // main page
        Constraint::Length(2), // footer
    ]).split(frame.area());

    match app.current_screen {
        CurrentScreen::Menu | CurrentScreen::NewPuzzlePopup | CurrentScreen::LoadPuzzlePopup | CurrentScreen::DeleteSavePopup => 
            render_title_content(frame, vertical_layout[0], app),
        CurrentScreen::Game | CurrentScreen::SavePuzzlePopup | CurrentScreen::ExitWarning => 
            render_game_board(frame, vertical_layout[0], app),
        CurrentScreen::Win => 
            render_win_screen(frame, vertical_layout[0], app),
    }
    render_footer(frame, vertical_layout[1], app);
    // render popup when appropriate
    match app.current_screen {
        CurrentScreen::NewPuzzlePopup => render_new_puzzle_popup(frame, app),
        CurrentScreen::SavePuzzlePopup => render_save_popup(frame),
        CurrentScreen::ExitWarning => render_exit_popup(frame),
        CurrentScreen::LoadPuzzlePopup => render_load_popup(frame, app),
        CurrentScreen::DeleteSavePopup => render_delete_save_popup(frame, app),
        _ => {}
    }
}

// LANDING PAGE

fn render_title_content(frame: &mut Frame, area: Rect, app: &App) {
    let center_area = center(
        area,
        Constraint::Length(60),
        Constraint::Length(20),
    );
    let container = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(Color::DarkGray));

    frame.render_widget(container.clone(), center_area);
    let inner_area = container.inner(center_area);
    let chunks = Layout::vertical([
        Constraint::Percentage(40), // title
        Constraint::Percentage(60), // buttons
    ])
    .split(inner_area);
    let title = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::new().fg(Color::Cyan))
        .lines(vec![
            "WSTUI".into()
        ])
        .alignment(ratatui::layout::Alignment::Center)
        .build();
    frame.render_widget(title, chunks[0]);

    let menu_buttons: Vec<Constraint> = app.menu_items.iter()
        .map(|_| Constraint::Length(1))
        .collect();
    let button_container = Layout::vertical(menu_buttons)
        .flex(Flex::Center)
        .spacing(1)
        .split(chunks[1]);
    for (i, item) in app.menu_items.iter().enumerate() {
        let label = match item {
            MenuItem::NewPuzzle => "New Puzzle",
            MenuItem::LoadPuzzle => "Load Puzzle",
            // MenuItem::Rules => "Rules",
            MenuItem::Quit => "Quit",
        };
        let style = if i == app.selected_item { Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::new().fg(Color::White) };
        let text = if i == app.selected_item {
            format!("> {} <", label)
        } else {
            label.to_string()
        };
        let btn = Paragraph::new(text).centered().style(style);
        frame.render_widget(btn, button_container[i]);
    }
}

// FOOTER

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mut block = Block::new()
        .borders(Borders::TOP)
        .border_style(Style::new().fg(Color::DarkGray));

    let line = Line::from(vec![
        Span::raw(" Nav: "),
        Span::styled("↑ ↓ ← →", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" / "),
        Span::styled("WASD", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | Select: "),
        Span::styled("Enter/Space", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | Scroll List: "),
        Span::styled("JK/Scroll", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled("V:", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Toggle found | "),
        Span::styled("G:", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Save | "),
        Span::styled("Esc:", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Back"),
    ]);
    if let (Some(msg), Some(time)) = (&app.notification, &app.notification_time) {
        if time.elapsed() < Duration::from_secs(3) {
            block = block.title_top(
                Line::from(Span::styled(
                    format!(" {} ", msg),
                    Style::default().fg(Color::Green)
                ))
                .alignment(Alignment::Left)
            );
        }
    }
    let paragraph = Paragraph::new(line)
        .block(block)
        .centered();
    frame.render_widget(paragraph, area);
}

// POPUPS

fn render_new_puzzle_popup(frame: &mut Frame, app: &App) {
    let area = center(
        frame.area(), 
        Constraint::Percentage(50),
        Constraint::Length(13)
    );
    frame.render_widget(Clear, area);
    let block = Block::bordered()
        .title(" Setup New Puzzle ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Reset)); 
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    let chunks = Layout::vertical([
        Constraint::Length(1), // top padding
        Constraint::Length(3), // puzzle type
        Constraint::Length(1), // padding
        Constraint::Length(3), // puzzle size
        Constraint::Length(2), // padding
        Constraint::Length(1), // buttons
    ]).split(inner_area);
    // button styles
    let active_pill_style = Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD);
    let inactive_text_style = Style::default().fg(Color::White);    
    // button group border styles
    let focused_border_style = Style::default().fg(Color::Yellow);
    let normal_border_style = Style::default().fg(Color::DarkGray);
    // PUZZLE TYPE
    let type_border_style = if app.popup_focus == PopupFocus::TypeSelector { focused_border_style } else { normal_border_style };
    let type_block = Block::bordered()
        .title(" Puzzle Type ")
        .title_alignment(Alignment::Center)
        .border_style(type_border_style);
    let type_inner = type_block.inner(chunks[1]);
    frame.render_widget(type_block, chunks[1]);
    let type_layout = Layout::horizontal(
        [Constraint::Percentage(50), Constraint::Percentage(50)]
    ).split(type_inner);
    let (style_2d, style_3d) = match app.puzzle_type {
        PuzzleType::TwoD => (active_pill_style, inactive_text_style),
        PuzzleType::ThreeD => (inactive_text_style, active_pill_style),
    };
    frame.render_widget(Paragraph::new("2D").centered().style(style_2d), type_layout[0]);
    frame.render_widget(Paragraph::new("3D").centered().style(style_3d), type_layout[1]);
    // PUZZLE SIZE
    let size_border_style = if app.popup_focus == PopupFocus::SizeSelector { focused_border_style } else { normal_border_style };
    let size_block = Block::bordered()
        .title(" Puzzle Size ")
        .title_alignment(Alignment::Center)
        .border_style(size_border_style);

    let size_inner = size_block.inner(chunks[3]);
    frame.render_widget(size_block, chunks[3]);

    let size_layout = Layout::horizontal([
        Constraint::Fill(1), 
        Constraint::Fill(1), 
        Constraint::Fill(1)
    ]).spacing(1).split(size_inner);
    let (style_s, style_m, style_l) = match app.puzzle_size {
        PuzzleSize::Small => (active_pill_style, inactive_text_style, inactive_text_style),
        PuzzleSize::Medium => (inactive_text_style, active_pill_style, inactive_text_style),
        PuzzleSize::Large => (inactive_text_style, inactive_text_style, active_pill_style),
    };
    frame.render_widget(Paragraph::new("Small").centered().style(style_s), size_layout[0]);
    frame.render_widget(Paragraph::new("Medium").centered().style(style_m), size_layout[1]);
    frame.render_widget(Paragraph::new("Large").centered().style(style_l), size_layout[2]);
    // BUTTONS
    let btn_area = center(chunks[5], Constraint::Length(30), Constraint::Length(1));
    let btn_layout = Layout::horizontal([
        Constraint::Percentage(50), 
        Constraint::Percentage(50)
    ]).spacing(2).split(btn_area);
    let labels = ["Start Game", "Cancel"];
    for (i, label) in labels.iter().enumerate() {
        let is_focused = app.popup_focus == PopupFocus::Buttons && app.popup_button_index == i;
        let style = if is_focused {
            active_pill_style
        } else {
            inactive_text_style
        };        
        frame.render_widget(Paragraph::new(label.to_string()).centered().style(style), btn_layout[i]);
    }
}

fn render_load_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_area = center(area, Constraint::Percentage(80), Constraint::Percentage(60));
    frame.render_widget(Clear, popup_area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Load Puzzle ")
        .title_alignment(Alignment::Center)
        // list controls in the border 
        .title_bottom(Line::from(vec![
            Span::raw(" [Enter] Load "),
            Span::raw(" [D] Delete "),
            Span::raw(" [Esc] Cancel "),
        ]).alignment(Alignment::Center))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White)); 
    frame.render_widget(block.clone(), popup_area);

    let inner_area = block.inner(popup_area);
    if app.save_files.is_empty() {
        let msg = Paragraph::new("No saved puzzles found.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD));
        let v_center = Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Fill(1)]).split(inner_area);
        frame.render_widget(msg, v_center[1]);
        return;
    }
    // header
    let header = Row::new(vec![
        "Date", "Type", "Size", "Progress", "Time"
    ])
    .style(Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))
    .bottom_margin(1);
    // table body
    let rows: Vec<Row> = app.save_files.iter().enumerate().map(|(i, save)| {
        let is_selected = i == app.selected_save_index;
        let puzzle_type = if save.puzzle_type == "TwoD" { "[2D]" } else { "[3D]" };
        let pct = (save.found as f64 / save.total as f64) * 100.0;
        let progress = format!("{}/{} ({:.0}%)", save.found, save.total, pct);
        let mm = save.elapsed / 60;
        let ss = save.elapsed % 60;
        let time = format!("{:02}:{:02}", mm, ss);
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        Row::new(vec![
            save.timestamp_str.clone(),
            puzzle_type.to_string(),
            save.puzzle_size.clone(),
            progress,
            time,
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Length(20), // Date
        Constraint::Length(8),  // Type
        Constraint::Length(8),  // Size
        Constraint::Length(15), // Progress
        Constraint::Length(10), // Time
    ];
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default());
    frame.render_widget(table, inner_area);
}

fn render_delete_save_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let popup_area = center(area, Constraint::Percentage(30), Constraint::Length(5));

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete Save ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Red).fg(Color::White));

    let filename = if !app.save_files.is_empty() {
        &app.save_files[app.selected_save_index].filename
    } else {
        "this file" // shouldnt happen
    };
    let text = vec![
        Line::from(format!("Delete {}?", filename)),
        Line::from(""),
        Line::from("(Y) Confirm   (N) Cancel"),
    ];
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, popup_area);
}

fn render_save_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup_area = center(area, Constraint::Percentage(30), Constraint::Length(5));
    
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Save Game ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Cyan).fg(Color::White));
    let text = vec![
        Line::from("Save current puzzle?"), 
        Line::from(""),
        Line::from("(Y) Confirm   (N) Cancel"),
    ];
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup_area);
}

fn render_exit_popup(frame: &mut Frame) {
    let area = frame.area();    
    let popup_area = center(area, Constraint::Percentage(30), Constraint::Length(6));

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" WARNING ")
        .style(Style::default().bg(Color::Red).fg(Color::White));
    let text = vec![
        Line::from("Exit? Unsaved progress"),
        Line::from("will be lost."), 
        Line::from(""),
        Line::from("(Y) Confirm   (N) Cancel"),
    ];
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, popup_area);
}

// PUZZLE RENDERING

fn render_game_board(frame: &mut Frame, area: Rect, app: &mut App) {
    let main_layout = Layout::horizontal([
        Constraint::Percentage(75), // grid
        Constraint::Percentage(25), // wordlist
    ]).split(area);
    let grid_container = main_layout[0];
    let list_container = main_layout[1];
    match &app.word_search {
        ActivePuzzle::TwoD(ws) => {
            if !ws.grid.is_empty() {
                render_2d_grid(frame, grid_container, ws, app);
            }
        },
        ActivePuzzle::ThreeD(ws) => {
            if !ws.grid.is_empty() {
                render_3d_grid(frame, grid_container, ws, app);
            }
        }
    }
    render_word_list(frame, list_container, app);
}

fn render_2d_grid(frame: &mut Frame, area: Rect, ws: &crate::ws_2d::puzzle::WordSearch, app: &App) {
    let grid_size = ws.grid.len();
    let col_width = 3;
    let required_width = ((grid_size * col_width) as u16)+2;
    let required_height = ((grid_size) as u16)+2;
    let centered_area = center(area, Constraint::Length(required_width), Constraint::Length(required_height));
    let selected_paths = app.get_current_selection_lines();
    let rows: Vec<Row> = ws.grid.iter().enumerate().map(|(x, row_vec)| {
        let cells: Vec<Cell> = row_vec.iter().enumerate()
            .map(|(y, c)| {
                let content = (*c as char).to_ascii_uppercase().to_string();
                let is_cursor = x == app.cursor_x && y == app.cursor_y;
                let is_found = app.found_cells.contains(&(x, y, 0));
                let path_idx = selected_paths.iter().position(|p| p.contains(&(x,y,0)));
                let mut style = Style::default().fg(Color::White);
                if let Some(idx) = path_idx {
                    // selection found
                    if idx == 0 {
                        // main line coloring
                        style = style.bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD);
                    } else {
                        // secondary line coloring
                        style = style.bg(Color::Magenta).fg(Color::Black).add_modifier(Modifier::BOLD);
                    }
                } else if is_cursor {
                    style = style.bg(Color::Cyan).fg(Color::White).add_modifier(Modifier::BOLD);
                } else if is_found && app.show_found_words {
                    style = style.bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD);
                }
                Cell::from(Line::from(content).alignment(Alignment::Center)).style(style)
            }).collect();
        Row::new(cells).height(1)
    }).collect();
    let col_constraints: Vec<Constraint> = (0..grid_size).map(|_| Constraint::Length(col_width as u16)).collect();
    let table = Table::new(rows, col_constraints)
        .block(Block::bordered().border_style(Style::default().fg(Color::DarkGray)))
        .column_spacing(0)
        .style(Style::default().fg(Color::White));
    frame.render_widget(table, centered_area);
}

fn render_3d_grid(frame: &mut Frame, area: Rect, ws: &ws_3d::puzzle::WordSearch, app: &App) {
    let width = area.width as f64;
    let height = area.height as f64;
    let n = ws.grid.get_size() as f64;
    // define isometric vectors
    let vx = (4.0, 1.0); 
    let vy = (-4.0, 1.0); 
    let vz = (0.0, -2.0); 
    let vertices = [
        (0.0, 0.0),
        (n * vx.0, n * vx.1),
        (n * vy.0, n * vy.1),
        (n * vx.0 + n * vy.0, n * vx.1 + n * vy.1),
        (n * vz.0, n * vz.1),
        (n * vx.0 + n * vz.0, n * vx.1 + n * vz.1),
        (n * vy.0 + n * vz.0, n * vy.1 + n * vz.1),
    ];
    let mut min_x = f64::MAX; let mut max_x = f64::MIN;
    let mut min_y = f64::MAX; let mut max_y = f64::MIN;
    for (x, y) in vertices {
        min_x = min_x.min(x); max_x = max_x.max(x);
        min_y = min_y.min(y); max_y = max_y.max(y);
    }
    let pad = 4.0; 
    let scale = ((width - pad) / (max_x - min_x)).min((height - pad) / (max_y - min_y));
    let bbox_center_x = (min_x + max_x) / 2.0;
    let bbox_center_y = (min_y + max_y) / 2.0;
    let screen_center_x = width / 2.0;
    let screen_center_y = height / 2.0;
    // projection function
    let project = |u: f64, v: f64, w: f64| -> (f64, f64) {
        let raw_x = u * vx.0 + v * vy.0 + w * vz.0;
        let raw_y = u * vx.1 + v * vy.1 + w * vz.1;
        (
            screen_center_x + (raw_x - bbox_center_x) * scale,
            screen_center_y + (raw_y - bbox_center_y) * scale
        )
    };
    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([0.0, width])
        .y_bounds([0.0, height])
        .paint(|ctx| {
            let mut draw_line_3d = |u1, v1, w1, u2, v2, w2, color| {
                let (x1, y1) = project(u1, v1, w1);
                let (x2, y2) = project(u2, v2, w2);
                ctx.draw(&CanvasLine { x1, y1, x2, y2, color });
            };
            let inactive_border = Color::White;
            let active_border = Color::Yellow;
            let grid_color = Color::DarkGray;
            // Z=0: Top, Z=1: Right, Z=2: Left
            let top_col = if app.cursor_z == 0 { active_border } else { inactive_border };
            let right_col = if app.cursor_z == 1 { active_border } else { inactive_border };
            let left_col = if app.cursor_z == 2 { active_border } else { inactive_border };
            // grid lines (won't always be accurate unless user zooms out)
            for i in 1..ws.grid.get_size() {
                let k = i as f64;
                // TOP (w=0)
                draw_line_3d(k, 0.0, 0.0, k, n, 0.0, grid_color);
                draw_line_3d(0.0, k, 0.0, n, k, 0.0, grid_color);
                // RIGHT (v=0)
                draw_line_3d(k, 0.0, 0.0, k, 0.0, n, grid_color);
                draw_line_3d(0.0, 0.0, k, n, 0.0, k, grid_color);
                // LEFT (u=0)
                draw_line_3d(0.0, k, 0.0, 0.0, k, n, grid_color);
                draw_line_3d(0.0, 0.0, k, 0.0, n, k, grid_color);
            }
            // face borders    
            // TOP 
            draw_line_3d(0.0, 0.0, 0.0, n, 0.0, 0.0, top_col); // Spine-Right
            draw_line_3d(0.0, 0.0, 0.0, 0.0, n, 0.0, top_col); // Spine-Left
            draw_line_3d(n, 0.0, 0.0, n, n, 0.0, top_col);     // Outer-Right
            draw_line_3d(0.0, n, 0.0, n, n, 0.0, top_col);     // Outer-Left
            // RIGHT
            if app.cursor_z == 1 {
                draw_line_3d(0.0, 0.0, 0.0, n, 0.0, 0.0, right_col); // Spine-Right
            }
            draw_line_3d(0.0, 0.0, 0.0, 0.0, 0.0, n, right_col); // Spine-Down
            draw_line_3d(n, 0.0, 0.0, n, 0.0, n, right_col);     // Outer-Vertical
            draw_line_3d(0.0, 0.0, n, n, 0.0, n, right_col);     // Outer-Bottom
            // LEFT
            if app.cursor_z == 2 {
                draw_line_3d(0.0, 0.0, 0.0, 0.0, n, 0.0, left_col); // Spine-Left
                draw_line_3d(0.0, 0.0, 0.0, 0.0, 0.0, n, left_col); // Spine-Down
            } else if app.cursor_z != 1 {
                draw_line_3d(0.0, 0.0, 0.0, 0.0, 0.0, n, left_col); 
            }
            draw_line_3d(0.0, n, 0.0, 0.0, n, n, left_col); // Outer-Vertical
            draw_line_3d(0.0, 0.0, n, 0.0, n, n, left_col); // Outer-Bottom
            // letters
            draw_face_top(ctx, ws, app, &project);
            draw_face_right(ctx, ws, app, &project);
            draw_face_left(ctx, ws, app, &project);
        });
    frame.render_widget(canvas, area);
}

// Render each face

fn draw_face_top<F>(ctx: &mut Context, ws: &ws_3d::puzzle::WordSearch, app: &App, project: &F) where F: Fn(f64, f64, f64) -> (f64, f64) {
    let z_idx = 0;
    let selected_paths = app.get_current_selection_lines();
    let dim = ws.grid.get_size();
    let face = ws.grid.get_face(Face::Top);
    for r in 0..dim {
        for c in 0..dim {
            // i=>n-1-j, j=>n-1-i
            let byte = face[dim-1-c][dim-1-r];
            let (px, py) = project(c as f64 + 0.5, r as f64 + 0.5, 0.0);
            draw_letter(ctx, byte, dim-1-c as usize, dim-1-r, z_idx, px, py, app, &selected_paths);
        }
    }
}

fn draw_face_right<F>(ctx: &mut Context, ws: &ws_3d::puzzle::WordSearch, app: &App, project: &F) where F: Fn(f64, f64, f64) -> (f64, f64) {
    let z_idx = 1;
    let selected_paths = app.get_current_selection_lines();
    let dim = ws.grid.get_size();
    let face = ws.grid.get_face(Face::Right);
    for r in 0..dim {
        for c in 0..dim {
            // i,j (unchanged)
            let byte = face[r][c];
            let (px, py) = project(c as f64 + 0.5, 0.0, r as f64 + 0.5);
            draw_letter(ctx, byte, r, c, z_idx, px, py, app, &selected_paths);
        }
    }
}

fn draw_face_left<F>(ctx: &mut Context, ws: &ws_3d::puzzle::WordSearch, app: &App, project: &F) where F: Fn(f64, f64, f64) -> (f64, f64) {
    let z_idx = 2;
    let selected_paths = app.get_current_selection_lines();
    let dim = ws.grid.get_size();
    let face = ws.grid.get_face(Face::Left);
    for r in 0..dim {
        for c in 00..dim {
            // i (unchanged), j=>n-1-j
            let byte = face[r][dim-1-c];
            let (px, py) = project(0.0, c as f64 + 0.5, r as f64 + 0.5);
            draw_letter(ctx, byte, r, dim-1-c, z_idx, px, py, app, &selected_paths);
        }
    }
}

fn draw_letter(ctx: &mut Context, byte: u8, r: usize, c: usize, z: usize, px: f64, py: f64, app: &App, selected_paths: &[Vec<(usize, usize, usize)>]) {
    let ch = (byte as char).to_ascii_uppercase();
    let is_cursor = app.cursor_z == z && app.cursor_x == r && app.cursor_y == c;
    let is_found = app.found_cells.contains(&(r, c, z));
    let path_idx = selected_paths.iter().position(|path| path.contains(&(r, c, z)));
    let mut style = Style::default();
    if let Some(idx) = path_idx {
        // selection found
        if idx == 0 {
            // main line coloring
            style = style.bg(Color::Blue).fg(Color::Black).add_modifier(Modifier::BOLD);
        } else {
            // secondary line coloring
            style = style.bg(Color::Magenta).fg(Color::Black).add_modifier(Modifier::BOLD);
        }
    } else if is_cursor {
        style = style.fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD);
    } else if is_found && app.show_found_words {
        style = style.fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD);
    } else {
        style = style.fg(Color::White);
    }
    ctx.print(px, py, Line::from(Span::styled(ch.to_string(), style)));
}

fn render_word_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let words = app.word_search.get_words();
    let total_words = words.len();
    let found_count = app.found_words.len();
    let remaining = total_words.saturating_sub(found_count);
    let list_height = area.height.saturating_sub(2) as usize;
    app.list_height = list_height; 
    let start_index = app.scroll_offset;
    let visible_words = words.iter()
        .skip(start_index)
        .take(list_height); 
    let word_items: Vec<ListItem> = visible_words
        .map(|w| {
            let is_found = app.found_words.contains(w);
            let style = if is_found {
                Style::default().fg(Color::Green).add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::Yellow)
            };
            ListItem::new(Line::from(vec![Span::styled(w.clone(), style)]))
        })
        .collect();
    let mut block = Block::bordered()
        .title(format!(" Words (Left: {}) ", remaining))
        .border_style(Style::default().fg(Color::Cyan));
    let show_up_arrow = start_index > 0;
    let show_down_arrow = (start_index + list_height) < total_words;
    if show_up_arrow {
        block = block.title_top(
            Line::from(
                Span::styled(" ↑ ", Style::default().fg(Color::DarkGray))
            )
            .alignment(Alignment::Right)
        );
    }
    if let Some(start) = app.start_time {
        let elapsed_secs = start.elapsed().as_secs();
        let mm = elapsed_secs / 60;
        let ss = elapsed_secs % 60;
        // show bottom arrow next to stopwatch if necessary
        // or else just show stopwatch 
        let content = if show_down_arrow { format!(" Time: {:02}:{:02} ↓ ", mm, ss) } else { format!(" Time: {:02}:{:02} ", mm, ss) };
        block = block.title_bottom(
            Line::from(Span::styled(
                content,
                Style::default().fg(Color::DarkGray)
            ))
            .alignment(Alignment::Right)
        );
    }
    let word_list = List::new(word_items).block(block);
    frame.render_widget(word_list, area);
}

fn render_win_screen(frame: &mut Frame, area: Rect, app: &App) {
    let area = center(area, Constraint::Percentage(60), Constraint::Length(15));    
    let block = Block::bordered()
        .border_type(BorderType::Double)
        .border_style(Style::new().fg(Color::Green))
        .title(" CONGRATULATIONS ")
        .title_alignment(Alignment::Center);

    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let chunks = Layout::vertical([
        Constraint::Length(2), // padding
        Constraint::Length(5), // "YOU WIN"
        Constraint::Length(2), // padding
        Constraint::Length(1), // time
        Constraint::Length(1), // exit controls
    ]).split(inner);
    let title = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::new().fg(Color::Green))
        .lines(vec![
            "YOU WIN!".into()
        ])
        .alignment(ratatui::layout::Alignment::Center)
        .build();
    frame.render_widget(title, chunks[1]);
    // display time (MM:SS)
    let elapsed = if let Some(final_time) = app.finish_time {
        final_time.as_secs()
    } else {
        0
    };
    let mm = elapsed / 60;
    let ss = elapsed % 60;
    let time_text = format!("Time Taken: {:02}:{:02}", mm, ss);
    let p_time = Paragraph::new(time_text)
        .centered()
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    frame.render_widget(p_time, chunks[3]);
    let footer = Paragraph::new("Press <Enter> to return to menu")
        .centered()
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, chunks[4]);
}

// HELPERS

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal]).flex(Flex::Center).areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}