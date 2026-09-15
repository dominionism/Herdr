use super::*;

#[path = "../shell/overlays.rs"]
mod overlays;
#[path = "../shell/sidebar.rs"]
pub(in crate::client::shell) mod sidebar;
#[path = "../shell/tabs.rs"]
mod tabs;

pub(super) use super::agent_sidebar::render_agent_panel;
pub(super) use super::aggregate_navigation::navigator_rows as client_navigator_rows;
pub(super) use overlays::{render_client_overlay, render_context_menu, render_global_menu};
pub(super) use sidebar::{render_collapsed_sidebar, render_sidebar, workspace_entries};
pub(super) use tabs::{render_tab_bar, tab_bar_status_width};

pub(in crate::client::shell) fn render_sidebar_background(
    buffer: &mut Buffer,
    area: Rect,
    palette: &Palette,
) {
    buffer.set_style(area, Style::default().bg(palette.sidebar_bg));
    let separator_x = area.right().saturating_sub(1);
    // Draw the divider at the left edge of its cell and give the rest of the cell the
    // pane background, so the sidebar ends exactly at the line. A centered "│" left
    // half a cell of sidebar beyond it, making right-hand margins wider than the left.
    for y in area.y..area.bottom() {
        if let Some(cell) = buffer.cell_mut((separator_x, y)) {
            cell.set_symbol("▏");
            cell.set_style(
                Style::default()
                    .fg(palette.surface_dim)
                    .bg(ratatui::style::Color::Reset),
            );
        }
    }
}

pub(super) fn render_mode_bar(
    buffer: &mut Buffer,
    pane_area: Rect,
    mode: ClientShellMode,
    copy_mode: Option<&ClientCopyModeState>,
    endpoint_error: Option<&str>,
    update_available: bool,
    keybinds: &LiveKeybindConfig,
    palette: &Palette,
) -> Option<Rect> {
    if (mode == ClientShellMode::Terminal && endpoint_error.is_none()) || pane_area.is_empty() {
        return None;
    }

    let bar = Rect::new(
        pane_area.x,
        pane_area.y + pane_area.height.saturating_sub(1),
        pane_area.width,
        1,
    );
    let base = Style::default().fg(palette.overlay0).bg(palette.panel_bg);
    for x in bar.x..bar.x + bar.width {
        buffer[(x, bar.y)].set_symbol(" ").set_style(base);
    }

    let key = Style::default()
        .fg(palette.accent)
        .bg(palette.panel_bg)
        .add_modifier(Modifier::BOLD);
    let mode_style = Style::default()
        .fg(match palette.panel_bg {
            ratatui::style::Color::Reset => palette.surface_dim,
            color => color,
        })
        .bg(if mode == ClientShellMode::Resize {
            palette.mauve
        } else {
            palette.accent
        })
        .add_modifier(Modifier::BOLD);
    let prefix = crate::config::format_key_combo(keybinds.prefix);
    let prefix_rhs = |bindings: &crate::config::ActionKeybinds| {
        bindings
            .prefix_rhs_label()
            .unwrap_or_else(|| "unset".to_owned())
    };

    let mut segments = Vec::<(String, Style)>::new();
    if let Some(error) = endpoint_error {
        segments.extend([
            (" ERROR ".to_owned(), mode_style),
            (format!(" {error}"), base),
        ]);
    } else {
        match mode {
            ClientShellMode::Prefix => {
                segments.extend([
                    (" PREFIX ".to_owned(), mode_style),
                    (" ".to_owned(), base),
                    ("esc".to_owned(), key),
                    (" cancel  ".to_owned(), base),
                    (prefix, key),
                    (" send prefix  ".to_owned(), base),
                    (prefix_rhs(&keybinds.keybinds.workspace_picker), key),
                    (" workspace nav  ".to_owned(), base),
                    (prefix_rhs(&keybinds.keybinds.help), key),
                    (" keybinds".to_owned(), base),
                ]);
            }
            ClientShellMode::Navigate => {
                segments.extend([
                    (" NAVIGATE ".to_owned(), mode_style),
                    (" esc back  ".to_owned(), base),
                    ("↑/↓".to_owned(), key),
                    (" workspace  ".to_owned(), base),
                    ("tab".to_owned(), key),
                    (" pane  ".to_owned(), base),
                    (prefix_rhs(&keybinds.keybinds.help), key),
                    (" keybinds".to_owned(), base),
                ]);
            }
            ClientShellMode::Resize => {
                segments.extend([
                    (" RESIZE ".to_owned(), mode_style),
                    ("  ".to_owned(), base),
                    ("h/l".to_owned(), key),
                    (" width  ".to_owned(), base),
                    ("j/k".to_owned(), key),
                    (" height  ".to_owned(), base),
                    ("esc".to_owned(), key),
                    (" done".to_owned(), base),
                ]);
            }
            ClientShellMode::Copy => {
                let copy_mode = copy_mode?;
                if let Some(prompt) = copy_mode.search_prompt.as_ref() {
                    let marker = match prompt.direction {
                        crate::api::schema::PaneCopySearchDirection::Forward => "/",
                        crate::api::schema::PaneCopySearchDirection::Backward => "?",
                    };
                    segments.extend([
                        (" COPY ".to_owned(), mode_style),
                        (" ".to_owned(), base),
                        (marker.to_owned(), key),
                        (
                            prompt.query.clone(),
                            Style::default().fg(palette.text).bg(palette.panel_bg),
                        ),
                        ("█".to_owned(), key),
                        ("  enter search  esc cancel".to_owned(), base),
                    ]);
                } else {
                    let select = if copy_mode.selection.is_some() {
                        "selecting"
                    } else {
                        "select"
                    };
                    let match_status = copy_mode
                        .search_current_global
                        .map(|current| format!(" {}/{}", current + 1, copy_mode.search_total))
                        .or_else(|| (!copy_mode.search_query.is_empty()).then(|| " 0/0".to_owned()))
                        .unwrap_or_default();
                    let (exit_keys, exit_label) =
                        if copy_mode.search_query.is_empty() && copy_mode.selection.is_none() {
                            ("q/esc", " exit")
                        } else {
                            ("esc", " clear  q exit")
                        };
                    segments.extend([
                        (" COPY ".to_owned(), mode_style),
                        (" ".to_owned(), base),
                        ("h/j/k/l w/b/e { }".to_owned(), key),
                        (" move  ".to_owned(), base),
                        ("/ ?".to_owned(), key),
                        (" search  ".to_owned(), base),
                        ("n/N".to_owned(), key),
                        (format!(" repeat{match_status}  "), base),
                        ("v/space".to_owned(), key),
                        (format!(" {select}  "), base),
                        ("y/enter".to_owned(), key),
                        (" copy  ".to_owned(), base),
                        (exit_keys.to_owned(), key),
                        (exit_label.to_owned(), base),
                    ]);
                }
            }
            ClientShellMode::Terminal => unreachable!(),
        }
    }

    let mut x = bar.x;
    let end = bar.x + bar.width;
    for (text, style) in segments {
        if x >= end {
            break;
        }
        let remaining = end - x;
        buffer.set_stringn(x, bar.y, &text, usize::from(remaining), style);
        x = x.saturating_add(
            u16::try_from(UnicodeWidthStr::width(text.as_str()))
                .unwrap_or(u16::MAX)
                .min(remaining),
        );
    }
    if update_available && mode == ClientShellMode::Navigate {
        let width = 13.min(bar.width);
        let area = Rect::new(bar.right().saturating_sub(width), bar.y, width, 1);
        buffer.set_style(area, Style::default().bg(palette.panel_bg));
        put_right_text(
            buffer,
            area,
            area.y,
            " update ready",
            Style::default()
                .fg(palette.accent)
                .bg(palette.panel_bg)
                .add_modifier(Modifier::BOLD),
        );
    }
    Some(bar)
}

pub(super) struct ShellRenderState<'a> {
    pub(super) endpoints: &'a [ClientShellEndpoint],
    pub(super) active_endpoint_id: &'a ClientEndpointId,
    pub(super) collapsed_endpoints: &'a HashSet<ClientEndpointId>,
    pub(super) collapsed_groups: &'a HashSet<String>,
    pub(super) workspace_scroll: &'a mut usize,
    pub(super) agent_scroll: &'a mut usize,
    pub(super) tab_scroll: &'a mut usize,
    pub(super) reveal_focused_workspace: &'a mut bool,
    pub(super) reveal_focused_tab: &'a mut bool,
    pub(super) sidebar_collapsed: bool,
    pub(super) sidebar_section_split: f32,
    pub(super) tab_drag_insert_index: Option<usize>,
    pub(super) selected_workspace_id: Option<&'a str>,
    pub(super) dragged_workspace_id: Option<&'a str>,
    pub(super) workspace_drop_indicator_row: Option<u16>,
}

pub(super) fn render_shell(
    buffer: &mut Buffer,
    layout: ClientShellLayout,
    snapshot: &ClientShellSnapshot,
    config: &ClientShellConfig,
    mut state: ShellRenderState<'_>,
) -> ShellHitMap {
    let mut hits = ShellHitMap::default();
    if layout.mobile_header.height > 0 {
        super::mobile::render_mobile_header(
            buffer,
            layout.mobile_header,
            snapshot,
            config,
            &mut hits,
        );
    }
    if layout.sidebar.width > 0 {
        if state.endpoints.len() > 1 {
            if state.sidebar_collapsed {
                super::endpoint_sidebar::render_collapsed(
                    buffer,
                    layout.sidebar,
                    config,
                    &mut state,
                    &mut hits,
                );
            } else {
                super::endpoint_sidebar::render_expanded(
                    buffer,
                    layout.sidebar,
                    Some(snapshot),
                    config,
                    &mut state,
                    &mut hits,
                );
            }
        } else if state.sidebar_collapsed {
            render_collapsed_sidebar(buffer, layout.sidebar, snapshot, config, &mut hits);
        } else {
            render_sidebar(
                buffer,
                layout.sidebar,
                snapshot,
                config,
                &mut state,
                &mut hits,
            );
        }
    }
    if layout.tab_bar.height > 0 {
        render_tab_bar(
            buffer,
            layout.tab_bar,
            snapshot,
            config,
            state.tab_scroll,
            state.reveal_focused_tab,
            state.tab_drag_insert_index,
            &mut hits,
        );
    }
    if !config.mouse_capture {
        hits.sidebar_divider = Rect::default();
        hits.sidebar_section_divider = Rect::default();
        hits.workspace_scrollbar = Rect::default();
        hits.agent_scrollbar = Rect::default();
        hits.agent_sort_toggle = Rect::default();
        hits.new_workspace = Rect::default();
        hits.machines.clear();
        hits.workspaces.clear();
        hits.agents.clear();
        hits.endpoint_agents.clear();
        hits.tab_scroll_left = Rect::default();
        hits.tab_scroll_right = Rect::default();
        hits.new_tab = Rect::default();
        hits.pane_splits.clear();
    }
    hits
}

pub(super) fn put_right_text(buffer: &mut Buffer, area: Rect, y: u16, text: &str, style: Style) {
    let width = display_width(text).min(area.width);
    put_text(
        buffer,
        area.right().saturating_sub(width),
        y,
        width,
        text,
        style,
    );
}

pub(super) fn put_segment(
    buffer: &mut Buffer,
    x: u16,
    y: u16,
    right: u16,
    text: &str,
    style: Style,
) -> u16 {
    let width = display_width(text).min(right.saturating_sub(x));
    put_text(buffer, x, y, width, text, style);
    x.saturating_add(width)
}

pub(super) fn put_text(buffer: &mut Buffer, x: u16, y: u16, width: u16, text: &str, style: Style) {
    if width == 0 || y >= buffer.area.bottom() || x >= buffer.area.right() {
        return;
    }
    buffer.set_stringn(x, y, text, width as usize, style);
}

pub(super) fn display_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text).min(u16::MAX as usize) as u16
}

// Nerd Font glyphs used by the sidebar's buttons.
pub(in crate::client::shell) const ICON_SIDEBAR_EXPANDED: &str = "\u{ebf3}";
pub(in crate::client::shell) const ICON_SIDEBAR_COLLAPSED: &str = "\u{ec02}";
pub(in crate::client::shell) const ICON_MENU: &str = "\u{f0c9}";
pub(in crate::client::shell) const ICON_PLUS: &str = "\u{f067}";
pub(in crate::client::shell) const ICON_CARET: &str = "\u{f0d7}";

/// Rows taken by an outlined sidebar button.
pub(in crate::client::shell) const BUTTON_HEIGHT: u16 = 3;
/// Width that makes an icon button look square despite tall terminal cells.
pub(in crate::client::shell) const ICON_BUTTON_WIDTH: u16 = 7;

/// Width of a button around `label`: one space and one border on each side,
/// never narrower than a square icon button.
pub(in crate::client::shell) fn button_width(label: &str) -> u16 {
    display_width(label)
        .saturating_add(4)
        .max(ICON_BUTTON_WIDTH)
}

/// Draws `label` centered in a rounded outline filling `area` and returns the
/// clickable area, or an empty rect when the button does not fit.
pub(in crate::client::shell) fn put_button(
    buffer: &mut Buffer,
    area: Rect,
    label: &str,
    style: Style,
    border: ratatui::style::Color,
) -> Rect {
    let clip = buffer.area;
    if area.width < 3
        || area.height < BUTTON_HEIGHT
        || area.right() > clip.right()
        || area.bottom() > clip.bottom()
    {
        return Rect::default();
    }
    put_rounded_box(buffer, area, clip, Style::default().fg(border));
    let label_width = display_width(label).min(area.width - 2);
    let label_x = area.x + (area.width - label_width) / 2;
    let label_y = area.y + area.height / 2;
    put_text(buffer, label_x, label_y, label_width, label, style);
    // Ghostty lets a Nerd Font icon followed by a space spread over two cells and
    // left-aligns it there, pushing it off the button's center. A following no-break
    // space is not whitespace to Ghostty, so the icon stays centered in one cell.
    let ends_with_icon = label
        .chars()
        .last()
        .is_some_and(|c| matches!(c, '\u{e000}'..='\u{f8ff}' | '\u{f0000}'..='\u{ffffd}'));
    let after_label = label_x + label_width;
    if ends_with_icon && after_label < area.right() - 1 {
        put_text(buffer, after_label, label_y, 1, "\u{a0}", style);
    }
    area
}

/// Draws a rounded border around `rect`, inset one column from each side, on the
/// rows directly above and below it. Rows outside `clip` are left untouched.
pub(in crate::client::shell) fn put_focus_border(
    buffer: &mut Buffer,
    rect: Rect,
    clip: Rect,
    color: ratatui::style::Color,
) {
    if rect.y == 0 || rect.width < 5 {
        return;
    }
    let outline = Rect::new(rect.x + 1, rect.y - 1, rect.width - 2, rect.height + 2);
    put_rounded_box(buffer, outline, clip, Style::default().fg(color));
}

/// Draws a rounded outline along the edges of `outline`, skipping rows outside
/// `clip`. Buttons and the focus border share it so every corner matches.
fn put_rounded_box(buffer: &mut Buffer, outline: Rect, clip: Rect, style: Style) {
    if outline.width < 2 || outline.height < 2 {
        return;
    }
    let visible = |y: u16| y >= clip.y && y < clip.bottom();
    let right = outline.right() - 1;
    let bottom = outline.bottom() - 1;
    let rule = "─".repeat(usize::from(outline.width - 2));
    for (y, open, close) in [(outline.y, "╭", "╮"), (bottom, "╰", "╯")] {
        if visible(y) {
            put_text(buffer, outline.x, y, 1, open, style);
            put_text(buffer, outline.x + 1, y, outline.width - 2, &rule, style);
            put_text(buffer, right, y, 1, close, style);
        }
    }
    for y in outline.y + 1..bottom {
        if visible(y) {
            put_text(buffer, outline.x, y, 1, "│", style);
            put_text(buffer, right, y, 1, "│", style);
        }
    }
}
