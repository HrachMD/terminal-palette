use std::io;

use palette::Hsv;
use rand::Rng;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use arboard::Clipboard;

use crate::widgets::{
    content::{hex2rgb, rgb2hsv},
    status_bar::StatusBar,
};
use crate::{
    margin,
    widgets::content::{ColorBlock, MainContent},
};

pub const HEX_CHARS: [char; 22] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C', 'D', 'E', 'F', '0', '1', '2', '3', '4', '5', '6',
    '7', '8', '9',
];

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CurrentPage {
    Main,
    TheorySelector,
    EditColor,
}

#[derive(Copy, Clone, Debug, PartialEq, EnumIter)]
pub enum ColorTheories {
    Analogous,
    Complementary,
    Triad,
    Tetrad,
    Hexad,
    Monochrome,
    Shadows,
    Lights,
    Neutrals,
}

pub struct App {
    pub counter: i8,

    pub clipboard: Clipboard,

    pub theory_selector_state: ListState,
    pub current_page: CurrentPage,
    pub current_color_theory: ColorTheories,

    pub title: &'static str,
    pub color_block_count: usize,

    pub color_blocks: [Option<ColorBlock>; 9],
    pub selected_block_id: usize,

    pub status_bar_msg: &'static str,

    pub edit_color_field: String,

    pub exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&*self, frame.area());

        let popup_area = Rect {
            x: frame.area().width / 3,
            y: frame.area().height * 2 / 5,
            width: frame.area().width / 3,
            height: frame.area().height / 4,
        };

        if self.current_page == CurrentPage::TheorySelector {
            // SETTINGS POPUP

            let popup_list_items: Vec<ListItem> = ColorTheories::iter()
                .map(|t| ListItem::new(format!("{:?}", t)))
                .collect();

            let popup_list = List::new(popup_list_items)
                .block(
                    Block::default()
                        .title(" Select Theory ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain),
                )
                .highlight_symbol(">");

            frame.render_widget(Clear, popup_area);
            frame.render_stateful_widget(popup_list, popup_area, &mut self.theory_selector_state);
        } else if self.current_page == CurrentPage::EditColor {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
                .split(popup_area);

            let block = Block::default()
                .title(" Edit Color ")
                .borders(Borders::ALL)
                .border_type(BorderType::Plain);

            frame.render_widget(block, popup_area);

            let (r, g, b) = hex2rgb(&self.edit_color_field);

            let par = Paragraph::new(format!(" Enter HEX: {}", &self.edit_color_field));
            let overview = Paragraph::new(Line::from("Overview:").add_modifier(Modifier::REVERSED))
                .block(Block::new().bg(Color::Rgb(r, g, b)));

            frame.render_widget(Clear, popup_area.inner(margin!(1, 1)));
            frame.render_widget(par, layout[0].inner(margin!(1, 1)));
            frame.render_widget(overview, layout[1].inner(margin!(1, 1)));
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.current_page {
            CurrentPage::Main => match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('q'), _) => self.exit(),
                (KeyCode::Left, _) => self.decrement_counter(),
                (KeyCode::Right, _) => self.increment_counter(),

                (KeyCode::Char('a'), _) if self.color_block_count < 9 => self.add_block(),
                (KeyCode::Char('d'), _) if self.color_block_count > 3 => self.del_block(),

                (KeyCode::Char('x'), _) => {
                    self.theory_selector_state.select_first();
                    self.current_page = CurrentPage::TheorySelector
                }

                (KeyCode::Char('z'), _) => {
                    self.current_page = CurrentPage::EditColor;
                }

                (KeyCode::Char('l'), _) => {
                    if let Some(array_idx) =
                        self.get_array_index_for_logical_position(self.selected_block_id)
                    {
                        if let Some(block) = self.color_blocks[array_idx].as_mut() {
                            block.locked = !block.locked;
                        }
                    }
                }

                (KeyCode::Char('c'), _) => {
                    if let Some(array_idx) =
                        self.get_array_index_for_logical_position(self.selected_block_id)
                    {
                        if let Some(block) = self.color_blocks[array_idx].as_ref() {
                            self.clipboard.set_text(block.get_hex()).unwrap();
                        }
                    }
                }

                (KeyCode::Char(c), KeyModifiers::ALT) if ('1'..='9').contains(&c) => {
                    let num = c.to_digit(10).unwrap() as usize;
                    self.toggle_lock(num);
                }

                (KeyCode::Char(' '), _) => match self.current_color_theory {
                    ColorTheories::Analogous => self.generate_analogous(),
                    ColorTheories::Complementary => self.generate_complementary(),
                    ColorTheories::Triad => self.generate_triad(),
                    ColorTheories::Tetrad => self.generate_tetrad(),
                    ColorTheories::Hexad => self.generate_hexad(),
                    ColorTheories::Monochrome => self.generate_monochrome(),
                    ColorTheories::Shadows => self.generate_shades(false),
                    ColorTheories::Lights => self.generate_shades(true),
                    ColorTheories::Neutrals => self.generate_neutrals(),
                },

                _ => {}
            },
            CurrentPage::TheorySelector => match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('x'), _) | (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                    self.current_page = CurrentPage::Main
                }

                (KeyCode::Left, _) => self.theory_selector_state.select_first(),
                (KeyCode::Right, _) => self.theory_selector_state.select_last(),
                (KeyCode::Up, _) => self.theory_selector_state.select_previous(),
                (KeyCode::Down, _) => self.theory_selector_state.select_next(),

                (KeyCode::Enter, _) | (KeyCode::Char(' '), _) => {
                    if let Some(selected) = self.theory_selector_state.selected() {
                        let theories: Vec<ColorTheories> = ColorTheories::iter().collect();
                        self.current_color_theory = theories[selected];
                        self.current_page = CurrentPage::Main;
                    }
                }

                _ => {}
            },

            CurrentPage::EditColor => match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('z'), _) | (KeyCode::Char('q'), _) => {
                    self.current_page = CurrentPage::Main
                }

                (KeyCode::Char(c), _)
                    if HEX_CHARS.contains(&c) && self.edit_color_field.len() < 6 =>
                {
                    self.edit_color_field.push(c);
                }

                // doesnt work gonna look later
                (KeyCode::Backspace, KeyModifiers::CONTROL) => {
                    self.edit_color_field = String::new();
                }

                (KeyCode::Backspace, _) => {
                    self.edit_color_field.pop();
                }

                (KeyCode::Enter, _) => {
                    if let Some(array_idx) =
                        self.get_array_index_for_logical_position(self.selected_block_id)
                    {
                        if let Some(block) = self.color_blocks[array_idx].as_mut() {
                            let (r, g, b) = hex2rgb(&self.edit_color_field);
                            let (h, s, v) = rgb2hsv(r, g, b);
                            block.hsv = Hsv::new(h, s, v);
                            self.edit_color_field = String::new();
                        }
                    }
                }

                _ => {}
            },
        }
    }

    fn get_locked_blocks(&mut self) -> Vec<Option<ColorBlock>> {
        self.color_blocks
            .iter()
            .filter(|block| block.is_some())
            .filter(|block| block.unwrap().locked)
            .cloned()
            .collect()
    }

    fn generate_tetrad(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 4; // Minimal randomness for cleaner tetrad relationships

        let mut base_sat: f32 = 0.68;
        let mut base_val: f32 = 0.63;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            base_sat = ColorBlock::get_avg_saturation(&locked_blocks);
            base_val = ColorBlock::get_avg_value(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
                base_sat = color_block.hsv.saturation;
                base_val = color_block.hsv.value;
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        let total_blocks = block_info.len();

        // Determine how many base colors we have (4 for tetrad)
        let base_colors = 4;
        let colors_per_group = (total_blocks + base_colors - 1) / base_colors; // Round up division

        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                // Determine which base color group (0, 1, 2, or 3 for tetrad)
                let color_group = *logical_pos % base_colors;
                let variation_index = *logical_pos / base_colors;

                // Calculate base hue for this group
                let group_base_hue = match color_group {
                    0 => base_hue,
                    1 => (base_hue + 90.0) % 360.0,
                    2 => (base_hue + 180.0) % 360.0,
                    3 => (base_hue + 270.0) % 360.0,
                    _ => unreachable!(),
                };

                // Create variations within each color group
                let variation_factor = if colors_per_group > 1 {
                    (variation_index as f32) / (colors_per_group - 1) as f32 // 0.0 to 1.0
                } else {
                    0.5
                };

                let new_hue = (group_base_hue + randomness) % 360.0;

                // Vary saturation and value to create distinct variations within each group
                let sat_variation_range = if !locked_blocks.is_empty() {
                    0.12 // Moderate variation when locked color exists
                } else {
                    0.16 // More variation when no locked color
                };
                let val_variation_range = if !locked_blocks.is_empty() {
                    0.15 // Moderate variation when locked color exists
                } else {
                    0.20 // More variation when no locked color
                };

                // Create variation: center around base, spread based on variation_index
                let sat_offset = (variation_factor - 0.5) * sat_variation_range * 2.0;
                let val_offset = (variation_factor - 0.5) * val_variation_range * 2.0;

                let new_sat = (base_sat + sat_offset).clamp(0.0, 1.0);
                let new_val = (base_val + val_offset).clamp(0.0, 1.0);

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_hexad(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 4; // Minimal randomness for cleaner hexad relationships

        let mut base_sat: f32 = 0.65;
        let mut base_val: f32 = 0.60;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            base_sat = ColorBlock::get_avg_saturation(&locked_blocks);
            base_val = ColorBlock::get_avg_value(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
                base_sat = color_block.hsv.saturation;
                base_val = color_block.hsv.value;
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        let total_blocks = block_info.len();

        // Determine how many base colors we have (6 for hexad)
        let base_colors = 6;
        let colors_per_group = (total_blocks + base_colors - 1) / base_colors; // Round up division

        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                // Determine which base color group (0-5 for hexad)
                let color_group = *logical_pos % base_colors;
                let variation_index = *logical_pos / base_colors;

                // Calculate base hue for this group
                let group_base_hue = match color_group {
                    0 => base_hue,
                    1 => (base_hue + 60.0) % 360.0,
                    2 => (base_hue + 120.0) % 360.0,
                    3 => (base_hue + 180.0) % 360.0,
                    4 => (base_hue + 240.0) % 360.0,
                    5 => (base_hue + 300.0) % 360.0,
                    _ => unreachable!(),
                };

                // Create variations within each color group (if more blocks than base colors)
                let variation_factor = if colors_per_group > 1 {
                    (variation_index as f32) / (colors_per_group - 1) as f32 // 0.0 to 1.0
                } else {
                    0.5
                };

                let new_hue = (group_base_hue + randomness) % 360.0;

                // Vary saturation and value to create distinct variations within each group
                let sat_variation_range = if !locked_blocks.is_empty() {
                    0.10 // Moderate variation when locked color exists
                } else {
                    0.14 // More variation when no locked color
                };
                let val_variation_range = if !locked_blocks.is_empty() {
                    0.12 // Moderate variation when locked color exists
                } else {
                    0.18 // More variation when no locked color
                };

                // Create variation: center around base, spread based on variation_index
                let sat_offset = (variation_factor - 0.5) * sat_variation_range * 2.0;
                let val_offset = (variation_factor - 0.5) * val_variation_range * 2.0;

                let new_sat = (base_sat + sat_offset).clamp(0.0, 1.0);
                let new_val = (base_val + val_offset).clamp(0.0, 1.0);

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_triad(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 4; // Minimal randomness for cleaner triadic relationships

        let mut base_sat: f32 = 0.72;
        let mut base_val: f32 = 0.68;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            base_sat = ColorBlock::get_avg_saturation(&locked_blocks);
            base_val = ColorBlock::get_avg_value(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
                base_sat = color_block.hsv.saturation;
                base_val = color_block.hsv.value;
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        let total_blocks = block_info.len();

        // Determine how many base colors we have (3 for triadic)
        let base_colors = 3;
        let colors_per_group = (total_blocks + base_colors - 1) / base_colors; // Round up division

        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                // Determine which base color group (0, 1, or 2 for triadic)
                let color_group = *logical_pos % base_colors;
                let variation_index = *logical_pos / base_colors;

                // Calculate base hue for this group
                let group_base_hue = match color_group {
                    0 => base_hue,
                    1 => (base_hue + 120.0) % 360.0,
                    2 => (base_hue + 240.0) % 360.0,
                    _ => unreachable!(),
                };

                // Create variations within each color group
                // Variation index determines how much to vary saturation/value
                let variation_factor = if colors_per_group > 1 {
                    (variation_index as f32) / (colors_per_group - 1) as f32 // 0.0 to 1.0
                } else {
                    0.5
                };

                let new_hue = (group_base_hue + randomness) % 360.0;

                // Vary saturation and value to create distinct variations within each group
                // Create a progression: lighter/darker or more/less saturated variations
                let sat_variation_range = if !locked_blocks.is_empty() {
                    0.12 // Moderate variation when locked color exists
                } else {
                    0.18 // More variation when no locked color
                };
                let val_variation_range = if !locked_blocks.is_empty() {
                    0.15 // Moderate variation when locked color exists
                } else {
                    0.22 // More variation when no locked color
                };

                // Create variation: center around base, spread based on variation_index
                let sat_offset = (variation_factor - 0.5) * sat_variation_range * 2.0; // -range to +range
                let val_offset = (variation_factor - 0.5) * val_variation_range * 2.0; // -range to +range

                let new_sat = (base_sat + sat_offset).clamp(0.0, 1.0);
                let new_val = (base_val + val_offset).clamp(0.0, 1.0);

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_complementary(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 4; // Minimal randomness for cleaner complementary relationships

        let mut base_sat: f32 = 0.70;
        let mut base_val: f32 = 0.65;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            base_sat = ColorBlock::get_avg_saturation(&locked_blocks);
            base_val = ColorBlock::get_avg_value(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
                base_sat = color_block.hsv.saturation;
                base_val = color_block.hsv.value;
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        let total_blocks = block_info.len();

        // Determine how many base colors we have (2 for complementary)
        let base_colors = 2;
        let colors_per_group = (total_blocks + base_colors - 1) / base_colors; // Round up division

        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                // Determine which base color group (0 = base, 1 = complement)
                let color_group = *logical_pos % base_colors;
                let variation_index = *logical_pos / base_colors;

                // Calculate base hue for this group
                let group_base_hue = if color_group == 0 {
                    base_hue
                } else {
                    (base_hue + 180.0) % 360.0
                };

                // Create variations within each color group
                // Variation index determines how much to vary saturation/value
                let variation_factor = if colors_per_group > 1 {
                    (variation_index as f32) / (colors_per_group - 1) as f32 // 0.0 to 1.0
                } else {
                    0.5
                };

                let new_hue = (group_base_hue + randomness) % 360.0;

                // Vary saturation and value to create distinct variations within each group
                // Create a progression: lighter/darker or more/less saturated variations
                let sat_variation_range = if !locked_blocks.is_empty() {
                    0.12 // Moderate variation when locked color exists
                } else {
                    0.18 // More variation when no locked color
                };
                let val_variation_range = if !locked_blocks.is_empty() {
                    0.15 // Moderate variation when locked color exists
                } else {
                    0.22 // More variation when no locked color
                };

                // Create variation: center around base, spread based on variation_index
                let sat_offset = (variation_factor - 0.5) * sat_variation_range * 2.0; // -range to +range
                let val_offset = (variation_factor - 0.5) * val_variation_range * 2.0; // -range to +range

                let new_sat = (base_sat + sat_offset).clamp(0.0, 1.0);
                let new_val = (base_val + val_offset).clamp(0.0, 1.0);

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_analogous(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let mut base_sat: f32 = 0.65;
        let mut base_val: f32 = 0.65;
        let rand_rate = 3; // Minimal randomness for cleaner analogous relationships

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            base_sat = ColorBlock::get_avg_saturation(&locked_blocks);
            base_val = ColorBlock::get_avg_value(&locked_blocks);
        } else {
            // generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
                base_sat = color_block.hsv.saturation;
                base_val = color_block.hsv.value;
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        let total_blocks = block_info.len();

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        // Best practice: analogous colors should stay within a reasonable range
        // to maintain true analogous harmony while having noticeable differences
        // Professional tools like palettegenerator.com distribute colors bidirectionally
        // Use a fixed step size for consistent, noticeable differences between colors
        let step_size = 10.0; // Fixed 10° step for clear, noticeable differences

        // Find the locked block's logical position to use as center (if any)
        let center_logical_pos = logical_positions
            .iter()
            .find(|(_, _, is_locked)| *is_locked)
            .map(|(_, logical_pos, _)| *logical_pos)
            .unwrap_or(total_blocks / 2); // Use middle if no locked block

        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                // Distribute colors bidirectionally around base hue
                // Colors before center go negative, colors after go positive
                let offset = if *logical_pos < center_logical_pos {
                    // Before center: negative offset
                    let diff = (center_logical_pos - *logical_pos) as f32;
                    -(diff * step_size)
                } else if *logical_pos > center_logical_pos {
                    // After center: positive offset
                    let diff = (*logical_pos - center_logical_pos) as f32;
                    diff * step_size
                } else {
                    // At center (shouldn't happen for unlocked, but safety)
                    0.0
                };
                let new_hue = ((base_hue + offset + randomness) % 360.0 + 360.0) % 360.0;

                // Vary saturation and value very slightly for visual interest while maintaining harmony
                // Analogous colors should stay very close to the base color's characteristics
                // Use locked blocks' saturation/value as base when available
                let sat_variation = if !locked_blocks.is_empty() {
                    0.05 // Very small variation when locked color exists (±5%)
                } else {
                    0.10 // Slightly more variation when no locked color (±10%)
                };
                let val_variation = if !locked_blocks.is_empty() {
                    0.05 // Very small variation when locked color exists (±5%)
                } else {
                    0.10 // Slightly more variation when no locked color (±10%)
                };

                let new_sat = (base_sat
                    + rng.random_range(-sat_variation..sat_variation) as f32 / 100.0)
                    .clamp(0.0, 1.0);
                let new_val = (base_val
                    + rng.random_range(-val_variation..val_variation) as f32 / 100.0)
                    .clamp(0.0, 1.0);

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_monochrome(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let hue_variation = 3.0; // Minimal hue variation for true monochrome (±3 degrees)
        let rand_rate = 2; // Very low randomness for hue to maintain monochromatic integrity

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            }
        }

        // Collect all existing blocks to calculate logical positions
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(_block) = block {
                block_info.push((i, _block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        let total_blocks = block_info.len();

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        // Get anchor saturation and value from locked blocks or first block
        let (anchor_sat, anchor_val) = if !locked_blocks.is_empty() {
            if let Some(Some(anchor_block)) = locked_blocks.first() {
                let (_, sat, val) = anchor_block.get_hsv_values();
                (sat, val)
            } else {
                (0.6, 0.6) // Default fallback
            }
        } else {
            if let Some(color_block) = self.color_blocks[0].as_ref() {
                let (_, sat, val) = color_block.get_hsv_values();
                (sat, val)
            } else {
                (0.6, 0.6) // Default fallback
            }
        };

        // For monochrome, we create variations in both saturation and brightness
        // This creates tints (lighter), tones (muted), and shades (darker)
        // Saturation range: from low (0.1) to high (0.9)
        // Brightness range: from low (0.2) to high (0.9)

        let sat_range_start = 0.1;
        let sat_range_end = 0.9;
        let val_range_start = 0.2;
        let val_range_end = 0.9;

        // Calculate step sizes for even distribution
        let sat_step = if total_blocks > 1 {
            (sat_range_end - sat_range_start) / (total_blocks - 1) as f32
        } else {
            0.0
        };

        let val_step = if total_blocks > 1 {
            (val_range_end - val_range_start) / (total_blocks - 1) as f32
        } else {
            0.0
        };

        // Apply monochrome progression to all unlocked blocks
        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                // Keep hue constant with minimal variation for true monochrome
                let hue_randomness = rng.random_range(-rand_rate..rand_rate) as f32;
                let new_hue = (base_hue + hue_randomness * hue_variation / 10.0) % 360.0;

                // Vary saturation across the range for visual interest
                // Create a smooth progression that doesn't necessarily follow anchor
                let new_sat = if locked_blocks.is_empty() {
                    // No locked blocks: distribute evenly across range
                    sat_range_start + (sat_step * *logical_pos as f32)
                } else {
                    // With locked blocks: use anchor saturation as reference but still vary
                    // Create variation around anchor while maintaining smooth progression
                    let base_sat_progress = sat_range_start + (sat_step * *logical_pos as f32);
                    // Blend with anchor saturation for smoother transitions
                    (base_sat_progress * 0.7 + anchor_sat * 0.3)
                        .clamp(sat_range_start, sat_range_end)
                };

                // Vary brightness across the range
                // Alternate between lighter and darker for more interesting palette
                let new_val = if locked_blocks.is_empty() {
                    // No locked blocks: distribute evenly across range
                    val_range_start + (val_step * *logical_pos as f32)
                } else {
                    // With locked blocks: use anchor value as reference but still vary
                    let base_val_progress = val_range_start + (val_step * *logical_pos as f32);
                    // Blend with anchor value for smoother transitions
                    (base_val_progress * 0.7 + anchor_val * 0.3)
                        .clamp(val_range_start, val_range_end)
                };

                color_block.change_color(new_hue, new_sat, new_val);
            }
        }
    }

    fn generate_shades(&mut self, to_light: bool) {
        // Full range: 0.0 (black) to 1.0 (white) - no constraints
        let black = 0.0;
        let white = 1.0;

        // Get base hue from locked blocks or generate
        let locked_blocks = self.get_locked_blocks();
        let base_hue: f32;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // Generate initial random color for first block if no locks
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            } else {
                return; // No blocks available
            }
        }

        // Collect all existing blocks with their array positions, values, saturations, and lock status
        // Then map them to logical positions (0, 1, 2, ...) for even distribution
        let mut block_info: Vec<(usize, f32, f32, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(block) = block {
                block_info.push((i, block.hsv.value, block.hsv.saturation, block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        let total_blocks = block_info.len();

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        // This ensures even distribution regardless of gaps in the array
        let mut logical_positions: Vec<(usize, usize, f32, f32, bool)> = Vec::new();
        for (logical_pos, (array_pos, val, sat, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *val, *sat, *is_locked));
        }

        // Find locked blocks and use the first one as anchor
        let locked_info: Vec<(usize, usize, f32, f32)> = logical_positions
            .iter()
            .filter_map(|(array_pos, logical_pos, val, sat, is_locked)| {
                if *is_locked {
                    Some((*array_pos, *logical_pos, *val, *sat))
                } else {
                    None
                }
            })
            .collect();

        // Determine anchor (first locked block, or first block if none)
        let (_anchor_array_pos, anchor_logical_pos, anchor_val, anchor_sat) =
            if let Some((_array_pos, logical_pos, val, sat)) = locked_info.first() {
                (*_array_pos, *logical_pos, *val, *sat)
            } else {
                // No locked blocks - use first block as anchor
                let (array_pos, logical_pos, val, sat, _) = logical_positions[0];
                (array_pos, logical_pos, val, sat)
            };

        // Calculate dynamic step size based on total block count
        // More blocks = smaller step (smoother transition)
        // Fewer blocks = larger step (bigger jumps)

        // Calculate how many blocks are after the anchor (including the anchor itself)
        let blocks_after_anchor = total_blocks - anchor_logical_pos;

        // Calculate progression from anchor to target
        // For Lights: anchor -> one step below white (evenly incremented)
        // For Shadows: anchor -> one step above black (evenly incremented)
        let step_from_anchor = if to_light {
            // Lights: target is one step below white
            // Calculate step size: (white - anchor_val) divided by number of blocks after anchor
            // This ensures even increments and last block is one step below white
            if blocks_after_anchor > 0 {
                (white - anchor_val) / blocks_after_anchor as f32
            } else {
                0.0
            }
        } else {
            // Shadows: target is one step above black
            // Calculate step size: anchor_val divided by number of blocks after anchor
            // This ensures even increments and last block is one step above black
            if blocks_after_anchor > 0 {
                anchor_val / blocks_after_anchor as f32
            } else {
                0.0
            }
        };

        // Calculate how many blocks are before the anchor
        let blocks_before_anchor = anchor_logical_pos;

        // Calculate step size from start to anchor (if there are blocks before)
        let step_to_anchor = if blocks_before_anchor > 0 {
            if to_light {
                // For lights: start is black, anchor is somewhere above
                (anchor_val - black) / blocks_before_anchor as f32
            } else {
                // For shadows: start is white, anchor is somewhere below
                (white - anchor_val) / blocks_before_anchor as f32
            }
        } else {
            0.0
        };

        // For Lights mode: calculate desaturation step (from anchor saturation to 0.0)
        // For Shadows mode: keep saturation constant (as requested - never change)
        let sat_step_from_anchor = if to_light && blocks_after_anchor > 1 {
            // Lights: desaturate from anchor_sat to 0.0 (white has no saturation)
            anchor_sat / (blocks_after_anchor - 1) as f32
        } else {
            0.0
        };

        let sat_step_to_anchor = if to_light && blocks_before_anchor > 0 {
            // Lights: before anchor, increase saturation from 0.0 to anchor_sat
            anchor_sat / blocks_before_anchor as f32
        } else {
            0.0
        };

        // Apply progression to all unlocked blocks
        for (array_pos, logical_pos, _current_val, _current_sat, is_locked) in
            logical_positions.iter()
        {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                // Calculate new value (brightness)
                let new_val = if *logical_pos < anchor_logical_pos {
                    // Before anchor: progress from start toward anchor
                    if to_light {
                        black + (step_to_anchor * *logical_pos as f32)
                    } else {
                        white - (step_to_anchor * *logical_pos as f32)
                    }
                } else if *logical_pos == anchor_logical_pos {
                    // At anchor: use anchor value (shouldn't happen for unlocked, but just in case)
                    anchor_val
                } else {
                    // After anchor: progress from anchor toward target
                    let steps_after = (*logical_pos - anchor_logical_pos) as f32;
                    if to_light {
                        anchor_val + (step_from_anchor * steps_after)
                    } else {
                        anchor_val - (step_from_anchor * steps_after)
                    }
                };

                // Clamp value to valid range [0.0, 1.0]
                let clamped_val = new_val.clamp(black, white);

                // Calculate new saturation
                let new_sat = if to_light {
                    // Lights mode: desaturate as we get lighter
                    if *logical_pos < anchor_logical_pos {
                        // Before anchor: increase saturation toward anchor
                        (sat_step_to_anchor * *logical_pos as f32).min(anchor_sat)
                    } else if *logical_pos == anchor_logical_pos {
                        anchor_sat
                    } else {
                        // After anchor: decrease saturation toward 0.0 (white)
                        let steps_after = (*logical_pos - anchor_logical_pos) as f32;
                        (anchor_sat - (sat_step_from_anchor * steps_after)).max(0.0)
                    }
                } else {
                    // Shadows mode: keep saturation constant (never change)
                    anchor_sat
                };

                color_block.change_color(base_hue, new_sat, clamped_val);
            }
        }
    }

    fn generate_neutrals(&mut self) {
        // Get base hue and anchor color from locked blocks or generate
        let locked_blocks = self.get_locked_blocks();
        let base_hue: f32;
        let anchor_sat: f32;
        let anchor_val: f32;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
            // Use the first locked block's saturation and value as anchor
            if let Some(Some(anchor_block)) = locked_blocks.first() {
                let (_, sat, val) = anchor_block.get_hsv_values();
                anchor_sat = sat;
                anchor_val = val;
            } else {
                return; // Should not happen, but safety check
            }
        } else {
            // Generate initial random color for first block if no locks
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                let (h, s, v) = color_block.get_hsv_values();
                base_hue = h;
                anchor_sat = s;
                anchor_val = v;
            } else {
                return; // No blocks available
            }
        }

        // Collect all existing blocks with their array positions and lock status
        let mut block_info: Vec<(usize, bool)> = Vec::new();
        for (i, block) in self.color_blocks.iter().enumerate() {
            if let Some(block) = block {
                block_info.push((i, block.locked));
            }
        }

        if block_info.is_empty() {
            return;
        }

        let total_blocks = block_info.len();

        // Map array positions to logical positions (0, 1, 2, ..., total_blocks-1)
        let mut logical_positions: Vec<(usize, usize, bool)> = Vec::new();
        for (logical_pos, (array_pos, is_locked)) in block_info.iter().enumerate() {
            logical_positions.push((*array_pos, logical_pos, *is_locked));
        }

        // Find locked blocks and use the first one as anchor
        let locked_info: Vec<(usize, usize)> = logical_positions
            .iter()
            .filter_map(|(array_pos, logical_pos, is_locked)| {
                if *is_locked {
                    Some((*array_pos, *logical_pos))
                } else {
                    None
                }
            })
            .collect();

        // Determine anchor logical position (first locked block, or first block if none)
        let anchor_logical_pos = if let Some((_, logical_pos)) = locked_info.first() {
            *logical_pos
        } else {
            // No locked blocks - use first block as anchor
            logical_positions[0].1
        };

        // Calculate desaturation progression
        // We'll create a smooth transition from anchor saturation to 0 (fully desaturated)
        // The anchor maintains its saturation, and other blocks desaturate progressively

        // Calculate how many blocks are after the anchor (including anchor)
        let blocks_after_anchor = total_blocks - anchor_logical_pos;

        // Calculate how many blocks are before the anchor
        let blocks_before_anchor = anchor_logical_pos;

        // Desaturation step: from anchor_sat to 0.0
        // Blocks before anchor: increase saturation from 0.0 to anchor_sat
        // Anchor: keep anchor_sat
        // Blocks after anchor: decrease saturation from anchor_sat to 0.0
        let sat_step_to_anchor = if blocks_before_anchor > 0 {
            anchor_sat / blocks_before_anchor as f32
        } else {
            0.0
        };

        let sat_step_from_anchor = if blocks_after_anchor > 1 {
            anchor_sat / (blocks_after_anchor - 1) as f32
        } else {
            0.0
        };

        // Apply neutral progression to all unlocked blocks
        for (array_pos, logical_pos, is_locked) in logical_positions.iter() {
            if *is_locked {
                continue; // Skip locked blocks
            }

            if let Some(color_block) = self.color_blocks[*array_pos].as_mut() {
                // Calculate new saturation (desaturation progression)
                let new_sat = if *logical_pos < anchor_logical_pos {
                    // Before anchor: increase saturation from 0.0 toward anchor
                    (sat_step_to_anchor * *logical_pos as f32).min(anchor_sat)
                } else if *logical_pos == anchor_logical_pos {
                    // At anchor: use anchor saturation (shouldn't happen for unlocked, but safety)
                    anchor_sat
                } else {
                    // After anchor: decrease saturation from anchor toward 0.0
                    let steps_after = (*logical_pos - anchor_logical_pos) as f32;
                    (anchor_sat - (sat_step_from_anchor * steps_after)).max(0.0)
                };

                // For neutrals, we keep the value relatively stable but add slight variation
                // for visual depth. This creates a more interesting neutral palette.
                // Value variation: ±5% from anchor value
                let value_variation = 0.05;
                let value_range = (anchor_val - value_variation).max(0.0)
                    ..=(anchor_val + value_variation).min(1.0);

                // Distribute value slightly across blocks for subtle depth
                let value_progress = if total_blocks > 1 {
                    (*logical_pos as f32) / ((total_blocks - 1) as f32)
                } else {
                    0.0
                };

                // Create a subtle value curve: slightly darker in middle, lighter at edges
                // This creates a more natural neutral palette
                let value_offset = (value_progress - 0.5) * 2.0; // -1.0 to 1.0
                let value_adjustment = value_offset * value_variation * 0.5; // Reduced variation
                let new_val =
                    (anchor_val + value_adjustment).clamp(*value_range.start(), *value_range.end());

                color_block.change_color(base_hue, new_sat, new_val);
            }
        }
    }

    fn get_existing_block_indices(&self) -> Vec<usize> {
        self.color_blocks
            .iter()
            .enumerate()
            .filter_map(|(idx, block)| if block.is_some() { Some(idx) } else { None })
            .collect()
    }

    fn get_array_index_for_logical_position(&self, logical_pos: usize) -> Option<usize> {
        let existing_blocks = self.get_existing_block_indices();
        existing_blocks.get(logical_pos).copied()
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        // Get actual count of existing blocks (not just count)
        let actual_count = self.color_blocks.iter().filter(|b| b.is_some()).count();
        if actual_count > 0 {
            self.selected_block_id = self
                .selected_block_id
                .saturating_add(1)
                .clamp(0, actual_count - 1);
        }
    }

    fn decrement_counter(&mut self) {
        // Get actual count of existing blocks (not just count)
        let actual_count = self.color_blocks.iter().filter(|b| b.is_some()).count();
        if actual_count > 0 {
            self.selected_block_id = self
                .selected_block_id
                .saturating_sub(1)
                .clamp(0, actual_count - 1);
        }
    }

    fn toggle_lock(&mut self, id: usize) {
        self.color_blocks[id - 1].as_mut().map(|color_block| {
            color_block.locked = !color_block.locked;
        });
    }

    fn add_block(&mut self) {
        if let Some(idx) = self.color_blocks.iter().position(|x| x.is_none()) {
            self.color_blocks[idx] = Some(ColorBlock::new(idx, 0 as f32, 0 as f32, 0 as f32));
            self.color_block_count += 1;
        }
    }

    fn del_block(&mut self) {
        if let Some(array_idx) = self.get_array_index_for_logical_position(self.selected_block_id) {
            // Delete the block
            self.color_blocks[array_idx] = None;
            self.color_block_count -= 1;

            // Adjust selected_block_id to stay within bounds
            let new_count = self.color_blocks.iter().filter(|b| b.is_some()).count();
            if new_count > 0 {
                self.selected_block_id = self.selected_block_id.min(new_count - 1);
            } else {
                self.selected_block_id = 0;
            }
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let color_block_count: usize = 5;
        let mut color_blocks: [Option<ColorBlock>; 9] = [None; 9];

        for i in 1..color_block_count + 1 {
            color_blocks[i - 1] = Some(ColorBlock::new(i, 0.0, 0.0, 0.0));
        }

        Self {
            counter: 0,

            clipboard: Clipboard::new().unwrap(),

            theory_selector_state: ListState::default(),
            current_page: CurrentPage::Main,
            current_color_theory: ColorTheories::Analogous,

            title: " Color Palette!!!!! ",
            color_block_count: color_block_count,
            selected_block_id: 0,

            color_blocks: color_blocks,

            status_bar_msg: "",

            edit_color_field: String::new(),

            exit: false,
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // SELECTED BLOCK
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(3)])
            .split(area);

        let (main_area, footer_area) = (layout[0], layout[1]);

        let mut main_content = MainContent::new(self.color_blocks, self.selected_block_id);
        main_content.render(main_area, buf);

        let status_bar = StatusBar::new(self.current_page);
        status_bar.render(footer_area, buf);
    }
}
