//! Push completed conversation turns into terminal scrollback above the viewport.
//!
//! Writes lines directly to the terminal's scrollback by printing them above
//! the viewport area. Uses newlines to scroll content upward into the terminal's
//! scrollback buffer.

use std::fmt;
use std::io;
use std::io::Write;

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Colors, Print, SetAttribute, SetColors};
use crossterm::terminal::{Clear, ClearType, ScrollUp};
use crossterm::Command;
use ratatui::backend::Backend;
use ratatui::layout::Size;
use ratatui::text::Line;

use crate::custom_terminal::CustomTerminal;

/// Insert styled lines into terminal scrollback above the viewport.
///
/// Strategy: scroll the entire screen up by N rows (pushing top rows into
/// terminal scrollback), then write the new content in the space created
/// above the viewport. The viewport position doesn't change relative to the
/// bottom of the screen.
pub fn insert_history_lines<B>(
    terminal: &mut CustomTerminal<B>,
    lines: Vec<Line>,
) -> io::Result<()>
where
    B: Backend + Write,
{
    if lines.is_empty() {
        return Ok(());
    }

    let screen_size = terminal.size().unwrap_or(Size::new(0, 0));
    if screen_size.height == 0 || screen_size.width == 0 {
        return Ok(());
    }

    let area = terminal.viewport_area;
    let wrap_width = area.width.max(1) as usize;

    // Count how many visual rows the content will take.
    let mut content_rows: u16 = 0;
    for line in &lines {
        let w = line.width().max(1);
        content_rows += (w.div_ceil(wrap_width)) as u16;
    }

    if content_rows == 0 {
        return Ok(());
    }

    let writer = terminal.backend_mut();

    // Scroll the entire screen up by content_rows. This:
    // 1. Pushes the top content_rows lines into terminal scrollback
    // 2. Creates content_rows blank lines at the bottom
    // 3. The viewport content shifts up, creating space at the bottom
    //
    // But we actually want space at the TOP (above viewport). So instead:
    // Use a scroll region from line 1 to viewport_top to insert content there.
    // If viewport is at row 0, we need to first scroll everything to make room.

    if area.top() == 0 {
        // Viewport fills from top of screen. Write history lines at the
        // top, then use ScrollUp to push them into terminal scrollback.

        // Write history at the top of the screen (overwrites viewport rows).
        queue!(writer, MoveTo(0, 0))?;
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                queue!(writer, Print("\r\n"))?;
            }
            queue!(writer, Clear(ClearType::UntilNewLine))?;
            write_line_spans(writer, line)?;
        }
        queue!(writer, SetAttribute(crossterm::style::Attribute::Reset))?;
        io::Write::flush(writer)?;

        // Push the history lines into scrollback.  ScrollUp moves the top
        // N rows off-screen into the terminal's scrollback buffer and
        // shifts the remaining screen content up.
        queue!(writer, ScrollUp(content_rows))?;
        io::Write::flush(writer)?;

        // Force full redraw since viewport content was shifted.
        terminal.clear()?;
    } else {
        // There's already space above the viewport. Use scroll region.
        queue!(writer, SetScrollRegion(1..area.top()))?;
        queue!(writer, MoveTo(0, area.top().saturating_sub(1)))?;

        for line in &lines {
            queue!(writer, Print("\r\n"))?;
            queue!(writer, Clear(ClearType::UntilNewLine))?;
            write_line_spans(writer, line)?;
        }

        queue!(writer, ResetScrollRegion)?;
        queue!(writer, SetAttribute(crossterm::style::Attribute::Reset))?;
        io::Write::flush(writer)?;

        terminal.clear()?;
    }

    Ok(())
}

fn write_line_spans(writer: &mut impl Write, line: &Line) -> io::Result<()> {
    use crossterm::style::Color as CColor;

    queue!(writer, SetAttribute(crossterm::style::Attribute::Reset))?;

    for span in &line.spans {
        let fg = span.style.fg.map(Into::into).unwrap_or(CColor::Reset);
        let bg = span.style.bg.map(Into::into).unwrap_or(CColor::Reset);
        queue!(writer, SetColors(Colors::new(fg, bg)))?;

        if span.style.add_modifier.contains(ratatui::style::Modifier::BOLD) {
            queue!(writer, SetAttribute(crossterm::style::Attribute::Bold))?;
        }
        if span.style.add_modifier.contains(ratatui::style::Modifier::DIM) {
            queue!(writer, SetAttribute(crossterm::style::Attribute::Dim))?;
        }
        if span.style.add_modifier.contains(ratatui::style::Modifier::ITALIC) {
            queue!(writer, SetAttribute(crossterm::style::Attribute::Italic))?;
        }

        queue!(writer, Print(span.content.as_ref()))?;
        queue!(writer, SetAttribute(crossterm::style::Attribute::Reset))?;
    }
    Ok(())
}

// --- ANSI scroll region commands ---

#[derive(Debug, Clone)]
pub struct SetScrollRegion(pub std::ops::Range<u16>);

impl Command for SetScrollRegion {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(f, "\x1b[{};{}r", self.0.start, self.0.end)
    }
    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ResetScrollRegion;

impl Command for ResetScrollRegion {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        write!(f, "\x1b[r")
    }
    #[cfg(windows)]
    fn execute_winapi(&self) -> io::Result<()> {
        Ok(())
    }
}
