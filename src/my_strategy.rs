use crate::debug_interface::DebugInterface;
// use ai_cup_22::debugging::Color;
use ai_cup_22::model::*;

pub struct MyStrategy {
    constants: Constants,
    pp: PotentialFields,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        let pp = PotentialFields::new(&constants);
        Self { constants, pp }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        // let debug_interface = debug_interface.unwrap();
        self.pp.calculate(game, &self.constants);
        let enemies: Vec<&Unit> = game
            .units
            .iter()
            .filter(|u| u.player_id != game.my_id)
            .collect();

        let me = game
            .units
            .iter()
            .filter(|u| u.player_id == game.my_id)
            .next()
            .unwrap();
        let closest_enemy = if me.weapon.is_none() {
            None // TODO:
        } else {
            enemies
                .iter()
                .filter(|e| e.position.square_distance(&me.position) < 1000.0)
                .min_by(|a, b| {
                    let a_dist = me.position.square_distance(&a.position);
                    let b_dist = me.position.square_distance(&b.position);
                    a_dist.partial_cmp(&b_dist).unwrap()
                })
        };

        let mut max_pp_value = -1.0;
        let mut max_pp_value_pos = Vec2::new(0.0, 0.0);
        let (x, y) = (me.position.x as i32, me.position.y as i32);
        for i in x - 5..x + 5 {
            for j in y - 5..y + 5 {
                let point = Vec2::new(i as f64, j as f64);
                let value = self.pp.value(point, closest_enemy.is_none());
                // debug_interface.add_circle(point, 0.3, color_by_value(value));
                if value > max_pp_value
                    || (value == max_pp_value
                        && max_pp_value_pos.square_distance(&me.position)
                            > point.square_distance(&me.position))
                {
                    max_pp_value = value;
                    max_pp_value_pos = point;
                }
            }
        }

        Order {
            unit_orders: game
                .units
                .iter()
                .filter(|u| u.player_id == game.my_id)
                .map(|me| {
                    let target_velocity = (max_pp_value_pos - me.position).normalize()
                        * self.constants.max_unit_forward_speed;
                    let target_direction = if let Some(closest_enemy) = closest_enemy {
                        closest_enemy.position + closest_enemy.velocity * 0.1 - me.position
                    } else {
                        if let Some(sound) = game
                            .sounds
                            .iter()
                            .filter(|s| s.position.square_distance(&me.position) < 1000.0)
                            .next()
                        {
                            sound.position - me.position
                        } else {
                            target_velocity
                        }
                    }
                    .normalize();
                    (
                        me.id,
                        UnitOrder {
                            target_velocity,
                            target_direction,
                            action: if let Some(closest_enemy) = closest_enemy {
                                let weapon_id = me.weapon.unwrap() as usize;
                                let weapon = &self.constants.weapons[weapon_id];
                                let mut aim = Line::new(
                                    me.position,
                                    me.position
                                        + (me.direction.normalize()
                                            * weapon.projectile_life_time
                                            * weapon.projectile_speed
                                            + self.constants.unit_radius * 2.0),
                                );
                                let d = closest_enemy.position.distance(&me.position);
                                if aim.length() > d + self.constants.unit_radius * 2.0 {
                                    aim.set_length(d + self.constants.unit_radius * 2.0)
                                }
                                // debug_interface.add_poly_line(
                                //     vec![aim.start, aim.end],
                                //     0.1,
                                //     Color::new(0.0, 0.0, 1.0, 0.5),
                                // );
                                // for o in self.constants.obstacles.iter().cloned() {
                                //     let circle = Circle::new(o.position, o.radius);
                                //     if circle.intercept_with_line(&aim) {
                                //         debug_interface.add_circle(
                                //             o.position,
                                //             o.radius,
                                //             Color::new(1.0, 0.0, 0.0, 0.5),
                                //         );
                                //     }
                                // }
                                let enemy_circle = Circle::new(
                                    closest_enemy.position + closest_enemy.velocity * 0.11,
                                    self.constants.unit_radius * 1.1,
                                );
                                let obstacle_on_line = self
                                    .constants
                                    .obstacles
                                    .iter()
                                    .filter(|o| !o.can_shoot_through)
                                    .any(|o| {
                                        let circle = Circle::new(o.position, o.radius + 0.1);
                                        circle.intercept_with_line(&aim)
                                    });

                                let my_weapon_range = self.constants.weapons[weapon_id]
                                    .projectile_speed
                                    * self.constants.weapons[me.weapon.unwrap() as usize]
                                        .projectile_life_time
                                    + self.constants.unit_radius * 2.0;

                                Some(ActionOrder::Aim {
                                    shoot: !obstacle_on_line
                                        && enemy_circle.intercept_with_line(&aim)
                                        && d <= my_weapon_range,
                                })
                            } else if let Some(loot) = game
                                .loot
                                .iter()
                                .filter(|l| {
                                    let t = &l.item;
                                    match t {
                                        Item::Weapon { type_index } => {
                                            if *type_index == 0 {
                                                return false;
                                            }
                                            if let Some(weapon_id) = me.weapon {
                                                if *type_index == weapon_id
                                                    || me.ammo[*type_index as usize] < 10
                                                {
                                                    return false;
                                                }
                                                let my_weapon =
                                                    &self.constants.weapons[weapon_id as usize];
                                                let new_weapon =
                                                    &self.constants.weapons[*type_index as usize];
                                                if my_weapon.projectile_life_time
                                                    * my_weapon.projectile_speed
                                                    > new_weapon.projectile_life_time
                                                        * new_weapon.projectile_speed
                                                {
                                                    return false;
                                                }
                                            } else {
                                                return true;
                                            }
                                        }
                                        _ => {}
                                    }
                                    l.position.distance(&me.position) < self.constants.unit_radius
                                })
                                .next()
                            {
                                Some(ActionOrder::Pickup { loot: loot.id })
                            } else if me.shield < self.constants.max_shield && me.shield_potions > 0
                            {
                                Some(ActionOrder::UseShieldPotion {})
                            } else {
                                None
                            },
                        },
                    )
                })
                .collect(),
        }
    }
    pub fn debug_update(&mut self, _displayed_tick: i32, _debug_interface: &mut DebugInterface) {}
    pub fn finish(&mut self) {}
}
