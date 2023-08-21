use ratatui::{style::{Style, Modifier, Color}, widgets::{Block, Borders, Widget}};
use tui_textarea::TextArea;

#[derive(Clone)]
pub struct FocusableTextArea<'a> {
    text_area: TextArea<'a>,
    pub focused: bool,
    focused_title: String,
    unfocused_title: String,
}

impl<'a> Default for FocusableTextArea<'a> {
    fn default() -> Self {
        Self::new([""], "Focused", "Unfocused")
    }
}

impl<'a> FocusableTextArea<'a> {
    
    pub fn new<I>(i: I, focused_title: impl Into<String>, unfocused_title: impl Into<String>) -> Self 
        where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        let mut focusable_text_area = Self {
            text_area: TextArea::new(i.into_iter().map(|s| s.into()).collect::<Vec<String>>()),
            focused: false,
            focused_title: focused_title.into(),
            unfocused_title: unfocused_title.into(),
        };

        focusable_text_area.set_unfocus_style();

        focusable_text_area
    }

    pub fn widget(&'a self) -> impl Widget + 'a {
        self.text_area.widget()
    }

    fn set_focus_style(&mut self) {
        self.text_area.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.text_area.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        self.text_area.set_block(
            Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightGreen))
            .title(self.focused_title.clone())
        );
    }

    fn set_unfocus_style(&mut self) {
        self.text_area.set_cursor_line_style(Style::default());
        self.text_area.set_cursor_style(Style::default());
        self.text_area.set_block(
            Block::default().borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray))
            .title(self.unfocused_title.clone())
        );
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
        if focused {
            self.set_focus_style();
        } else {
            self.set_unfocus_style();
        }
    }


    pub fn input(&mut self, input: impl Into<tui_textarea::Input>) {
        if self.focused {
            self.text_area.input(input);
        }
    }

    pub fn lines(&'a self) -> &'a[String] {
        self.text_area.lines()
    }
}