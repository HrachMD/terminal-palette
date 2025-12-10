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

#[derive(Debug, PartialEq)]
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
    Square,
    Shadows,
    Lights,
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
                    ColorTheories::Square => self.generate_square(),
                    ColorTheories::Shadows => self.generate_shades(false),
                    ColorTheories::Lights => self.generate_shades(true),
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

    fn generate_square(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 8; // Lower randomness for cleaner square relationships

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            }
        }

        for (i, block) in self.color_blocks.iter_mut().enumerate() {
            if let Some(color_block) = block {
                if !color_block.locked {
                    let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                    // Create square colors: base, base+90°, base+180°, base+270°
                    let new_hue = match i % 4 {
                        0 => (base_hue + randomness) % 360.0,         // Primary
                        1 => (base_hue + 90.0 + randomness) % 360.0,  // First square
                        2 => (base_hue + 180.0 + randomness) % 360.0, // Complement
                        3 => (base_hue + 270.0 + randomness) % 360.0, // Second square
                        _ => unreachable!(),
                    };

                    let new_sat = if locked_blocks.is_empty() {
                        rng.random_range(55..80) as f32 / 100.0 // Balanced saturation for square harmony
                    } else {
                        color_block.hsv.saturation
                    };

                    let new_val = if locked_blocks.is_empty() {
                        rng.random_range(50..75) as f32 / 100.0
                    } else {
                        color_block.hsv.value
                    };

                    color_block.change_color(new_hue, new_sat, new_val);
                }
            }
        }
    }

    fn generate_triad(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 8; // Lower randomness for cleaner triadic relationships

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            }
        }

        for (i, block) in self.color_blocks.iter_mut().enumerate() {
            if let Some(color_block) = block {
                if !color_block.locked {
                    let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                    // Create triadic colors: base, base+120°, base+240°
                    let new_hue = match i % 3 {
                        0 => (base_hue + randomness) % 360.0,         // Primary
                        1 => (base_hue + 120.0 + randomness) % 360.0, // First triad
                        2 => (base_hue + 240.0 + randomness) % 360.0, // Second triad
                        _ => unreachable!(),
                    };

                    let new_sat = if locked_blocks.is_empty() {
                        rng.random_range(60..85) as f32 / 100.0 // Slightly higher saturation for vibrant triads
                    } else {
                        color_block.hsv.saturation
                    };

                    let new_val = if locked_blocks.is_empty() {
                        rng.random_range(55..80) as f32 / 100.0
                    } else {
                        color_block.hsv.value
                    };

                    color_block.change_color(new_hue, new_sat, new_val);
                }
            }
        }
    }

    fn generate_complementary(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let rand_rate = 15;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // Generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            }
        }

        for (i, block) in self.color_blocks.iter_mut().enumerate() {
            if let Some(color_block) = block {
                if !color_block.locked {
                    let randomness = rng.random_range(-rand_rate..rand_rate) as f32;

                    // Alternate between base hue and its complement
                    let new_hue = if i % 2 == 0 {
                        (base_hue + randomness) % 360.0
                    } else {
                        (base_hue + 180.0 + randomness) % 360.0
                    };

                    let new_sat = if locked_blocks.is_empty() {
                        rng.random_range(50..80) as f32 / 100.0
                    } else {
                        color_block.hsv.saturation
                    };

                    let new_val = if locked_blocks.is_empty() {
                        rng.random_range(50..80) as f32 / 100.0
                    } else {
                        color_block.hsv.value
                    };

                    color_block.change_color(new_hue, new_sat, new_val);
                }
            }
        }
    }

    fn generate_analogous(&mut self) {
        let mut rng = rand::rng();
        let locked_blocks = self.get_locked_blocks();
        let mut base_hue: f32 = 0.0;
        let hue_step = 30.0; // standard analogous step
        let rand_rate = 10;

        if !locked_blocks.is_empty() {
            base_hue = ColorBlock::get_avg_hue(&locked_blocks);
        } else {
            // generate initial random color for first block
            if let Some(color_block) = self.color_blocks[0].as_mut() {
                color_block.generate_random_color();
                base_hue = color_block.hsv.hue.into_degrees();
            }
        }

        for (i, block) in self.color_blocks.iter_mut().enumerate() {
            if let Some(color_block) = block {
                if !color_block.locked {
                    let randomness = rng.random_range(-rand_rate..rand_rate) as f32;
                    let new_hue = (base_hue + (i as f32 * hue_step) + randomness) % 360.0;

                    let new_sat = if locked_blocks.is_empty() {
                        rng.random_range(50..80) as f32 / 100.0
                    } else {
                        color_block.hsv.saturation
                    };

                    let new_val = if locked_blocks.is_empty() {
                        rng.random_range(50..80) as f32 / 100.0
                    } else {
                        color_block.hsv.value
                    };

                    color_block.change_color(new_hue, new_sat, new_val);
                }
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

        let status_bar = StatusBar::default();
        status_bar.render(footer_area, buf);
    }
}
