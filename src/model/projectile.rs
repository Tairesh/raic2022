use super::*;

/// Weapon projectile
#[derive(Clone, Debug)]
pub struct Projectile {
    /// Unique id
    pub id: i32,
    /// Index of the weapon this projectile was shot from (starts with 0)
    pub weapon_type_index: i32,
    /// Id of unit who made the shot
    pub shooter_id: i32,
    /// Id of player (team), whose unit made the shot
    pub shooter_player_id: i32,
    /// Current position
    pub position: model::Vec2,
    /// Projectile's velocity
    pub velocity: model::Vec2,
    /// Left time of projectile's life
    pub life_time: f64,
}

impl Projectile {
    pub fn range(&self) -> f64 {
        self.moving_vec().length()
    }

    pub fn moving_vec(&self) -> Vec2 {
        self.velocity * self.life_time
    }

    pub fn as_line(&self) -> Line {
        Line::new(self.position, self.position + self.moving_vec())
    }

    pub fn is_dangerous(&self, me: &Unit, constants: &Constants) -> bool {
        if self.shooter_player_id == me.player_id && !constants.friendly_fire {
            // Это моя пуля и френдли фаер выключен
            return false;
        }
        let line = self.as_line();
        let distance = line.distance_to_point(&me.position);
        // TODO: проверить что перед этим она не попадёт в обстакл (не в юнита потому что он может отойти
        // пролетит близко к моему юниту
        distance < constants.unit_radius * 3.0
    }
}

impl trans::Trans for Projectile {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.id.write_to(writer)?;
        self.weapon_type_index.write_to(writer)?;
        self.shooter_id.write_to(writer)?;
        self.shooter_player_id.write_to(writer)?;
        self.position.write_to(writer)?;
        self.velocity.write_to(writer)?;
        self.life_time.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let id: i32 = trans::Trans::read_from(reader)?;
        let weapon_type_index: i32 = trans::Trans::read_from(reader)?;
        let shooter_id: i32 = trans::Trans::read_from(reader)?;
        let shooter_player_id: i32 = trans::Trans::read_from(reader)?;
        let position: model::Vec2 = trans::Trans::read_from(reader)?;
        let velocity: model::Vec2 = trans::Trans::read_from(reader)?;
        let life_time: f64 = trans::Trans::read_from(reader)?;
        Ok(Self {
            id,
            weapon_type_index,
            shooter_id,
            shooter_player_id,
            position,
            velocity,
            life_time,
        })
    }
}
