use h5t_core::Combatant;
use ratatui::prelude::*;

/// A widget to display a creature's hit points, changing color based on the current hit points.
#[derive(Debug)]
pub struct HitPoints {
    /// The current hit points.
    pub current: i32,

    /// The maximum hit points.
    pub max: i32,
}

impl HitPoints {
    /// Create a new [`HitPoints`] widget from a [`Combatant`].
    pub fn new(combatant: &Combatant) -> Self {
        Self {
            current: combatant.hit_points,
            max: combatant.max_hit_points(),
        }
    }

    /// Creates a [`Line`] widget containing the hit points display.
    pub fn line(&self) -> Line<'static> {
        let hp_color = Color::Rgb(
            (255.0 - self.current as f32 / self.max as f32 * 255.0) as u8,
            (self.current as f32 / self.max as f32 * 255.0) as u8,
            0,
        );

        Line::from(vec![
            Span::styled(format!("{}", self.current), hp_color),
            Span::raw(format!("/{}", self.max)),
        ])
    }
}

impl Widget for HitPoints {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}
