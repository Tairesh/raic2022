use super::*;

/// Sound heard by one of your units
#[derive(Clone, Debug)]
pub struct Sound {
    /// Sound type index (starting with 0)
    pub type_index: i32,
    /// Id of unit that heard this sound
    pub unit_id: i32,
    /// Position where sound was heard (different from sound source position)
    pub position: model::Vec2,
}

impl Sound {
    pub fn get_weapon_shooting_range(&self, constants: &Constants) -> f64 {
        let props = &constants.sounds[self.type_index as usize];
        match props.name.as_str() {
            "Wand" => constants.weapons[WAND as usize].range(),
            "Staff" => constants.weapons[STAFF as usize].range(),
            "Bow" => constants.weapons[BOW as usize].range(),
            _ => panic!("Unknown weapon type: {}", props.name),
        }
    }
}

impl trans::Trans for Sound {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.type_index.write_to(writer)?;
        self.unit_id.write_to(writer)?;
        self.position.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let type_index: i32 = trans::Trans::read_from(reader)?;
        let unit_id: i32 = trans::Trans::read_from(reader)?;
        let position: model::Vec2 = trans::Trans::read_from(reader)?;
        Ok(Self {
            type_index,
            unit_id,
            position,
        })
    }
}
