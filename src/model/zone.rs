use super::*;

/// Current state of the game zone
#[derive(Clone, Debug)]
pub struct Zone {
    /// Current center
    pub current_center: model::Vec2,
    /// Current radius
    pub current_radius: f64,
    /// Next center
    pub next_center: model::Vec2,
    /// Next radius
    pub next_radius: f64,
}

impl Default for Zone {
    fn default() -> Self {
        Self {
            current_center: Vec2::zero(),
            current_radius: 0.0,
            next_center: Vec2::zero(),
            next_radius: 0.0,
        }
    }
}

impl trans::Trans for Zone {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.current_center.write_to(writer)?;
        self.current_radius.write_to(writer)?;
        self.next_center.write_to(writer)?;
        self.next_radius.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let current_center: model::Vec2 = trans::Trans::read_from(reader)?;
        let current_radius: f64 = trans::Trans::read_from(reader)?;
        let next_center: model::Vec2 = trans::Trans::read_from(reader)?;
        let next_radius: f64 = trans::Trans::read_from(reader)?;
        Ok(Self {
            current_center,
            current_radius,
            next_center,
            next_radius,
        })
    }
}
