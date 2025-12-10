use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::app::CurrentPage;

#[derive(Debug)]
pub struct StatusBar {
    pub current_page: CurrentPage,
}

impl StatusBar {
    pub fn new(current_page: CurrentPage) -> Self {
        Self { current_page }
    }

    fn get_hints(&self) -> Vec<Span<'_>> {
        match self.current_page {
            CurrentPage::Main => vec![
                Span::styled("[q]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Quit  "),
                Span::styled("[←]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[→]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Move  "),
                Span::styled("[a]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Add  "),
                Span::styled("[d]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Delete  "),
                Span::styled("[x]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Theory  "),
                Span::styled("[z]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Edit  "),
                Span::styled("[l]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Lock  "),
                Span::styled("[c]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Copy  "),
                Span::styled("[Space]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Generate"),
            ],
            CurrentPage::TheorySelector => vec![
                Span::styled("[x]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[q]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[Esc]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Close  "),
                Span::styled("[←]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" First  "),
                Span::styled("[→]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Last  "),
                Span::styled("[↑]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[↓]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Move  "),
                Span::styled("[Enter]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[Space]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Apply"),
            ],
            CurrentPage::EditColor => vec![
                Span::styled("[z]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::styled("[q]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Cancel  "),
                Span::styled("[Backspace]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Delete  "),
                Span::styled("[Ctrl+Backspace]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Clear  "),
                Span::styled("[Enter]", Color::Cyan).add_modifier(Modifier::BOLD),
                Span::raw(" Apply"),
            ],
        }
    }
}

impl Widget for &StatusBar {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let block = Block::default()
            .bg(Color::Black)
            .padding(Padding::new(0, 0, 1, 1));

        let hints = self.get_hints();
        Paragraph::new(Line::from(hints))
            .alignment(Alignment::Center)
            .block(block)
            .render(area, buf);
    }
}
