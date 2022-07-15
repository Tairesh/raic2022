use super::*;

pub static WAND: i32 = 0;
pub static STAFF: i32 = 1;
pub static BOW: i32 = 2;

static PREFERRED_WEAPON: i32 = BOW;

/// Loot lying on the ground
#[derive(Clone, Debug)]
pub struct Loot {
    /// Unique id
    pub id: i32,
    /// Position
    pub position: model::Vec2,
    /// Item
    pub item: model::Item,
}

impl Loot {
    pub fn is_useful_to_me(&self, me: &Unit, constants: &Constants) -> bool {
        let item = &self.item;
        match item {
            &Item::Weapon { type_index } => {
                if me.weapon.is_none() {
                    return true;
                }

                let my_weapon = me.weapon.unwrap();
                if type_index != PREFERRED_WEAPON {
                    return me.ammo[my_weapon as usize] == 0 && me.ammo[type_index as usize] > 0;
                }

                if my_weapon == PREFERRED_WEAPON {
                    return false;
                }

                me.ammo[my_weapon as usize] > 0
            }
            &Item::Ammo {
                weapon_type_index, ..
            } => {
                if me.ammo[weapon_type_index as usize]
                    == constants.weapons[weapon_type_index as usize].max_inventory_ammo
                {
                    return false;
                }

                if let Some(my_weapon) = me.weapon {
                    if my_weapon == PREFERRED_WEAPON {
                        weapon_type_index == PREFERRED_WEAPON
                    } else {
                        weapon_type_index != STAFF
                    }
                } else {
                    true
                }
            }
            Item::ShieldPotions { .. } => {
                me.shield_potions != constants.max_shield_potions_in_inventory
            }
        }
    }
}

impl trans::Trans for Loot {
    fn write_to(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
        self.id.write_to(writer)?;
        self.position.write_to(writer)?;
        self.item.write_to(writer)?;
        Ok(())
    }
    fn read_from(reader: &mut dyn std::io::Read) -> std::io::Result<Self> {
        let id: i32 = trans::Trans::read_from(reader)?;
        let position: model::Vec2 = trans::Trans::read_from(reader)?;
        let item: model::Item = trans::Trans::read_from(reader)?;
        Ok(Self { id, position, item })
    }
}
