use crate::debug_interface::DebugInterface;
use ai_cup_22::model::*;
use ai_cup_22::potential_field::PotentialField;
use std::f64::consts::PI;

pub struct MyStrategy {
    constants: Constants,
    pp2: PotentialField,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        let pp = PotentialField::new(&constants);
        Self { constants, pp2: pp }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        // let debug_interface = _debug_interface.unwrap();
        self.pp2.update(game);

        // for sound in game.sounds.iter() {
        //     debug_interface.add_circle(sound.position, 1.0, Color::new(0.0, 1.0, 0.0, 0.8));
        // }
        //
        // for projectile in game.projectiles.iter() {
        //     let line = projectile.as_line();
        //     debug_interface.add_poly_line(
        //         vec![line.start, line.end],
        //         0.3,
        //         Color::new(1.0, 0.0, 0.0, 0.5),
        //     );
        // }

        Order {
            unit_orders: game
                .units
                .iter()
                .filter(|u| u.player_id == game.my_id)
                .map(|me| {
                    // for point in self.pp2.points_around(me.id).iter() {
                    //     let value = self.pp2.value(point);
                    //     debug_interface.add_circle(*point, 0.5, color_by_value(value));
                    //     debug_interface.add_placed_text(
                    //         *point,
                    //         format!("{:.2}", value),
                    //         Vec2::new(0.5, 0.5),
                    //         0.2,
                    //         Color::BLACK,
                    //     );
                    // }

                    let target_velocity = if self.pp2.is_in_danger(me) {
                        (*self
                            .pp2
                            .points_around(me.id)
                            .iter()
                            .max_by(|a, b| {
                                let a_value = self.pp2.value(a);
                                let b_value = self.pp2.value(b);
                                a_value.partial_cmp(&b_value).unwrap()
                            })
                            .unwrap()
                            - me.position)
                            .normalize()
                            * self.constants.max_unit_forward_speed
                    } else {
                        // TODO: запоминать бонусы
                        let bonus = game
                            .loot
                            .iter()
                            .filter(|l| {
                                l.is_useful_to_me(me, &self.constants)
                                    && l.position.distance(&game.zone.current_center)
                                        < game.zone.current_radius
                                            - self.constants.unit_radius * 2.0
                            })
                            .min_by(|l, r| {
                                l.position
                                    .square_distance(&me.position)
                                    .partial_cmp(&r.position.square_distance(&me.position))
                                    .unwrap()
                            });

                        let target_position = if let Some(bonus) = bonus {
                            bonus.position
                        } else {
                            me.position + me.direction.rotate(PI / 10.0) * 5.0
                        };

                        (target_position - me.position).normalize()
                            * self.constants.max_unit_forward_speed
                    };

                    // TODO: запоминать врагов
                    let closest_enemy =
                        if me.weapon.is_some() && me.ammo[me.weapon.unwrap() as usize] > 0 {
                            game.units
                                .iter()
                                .filter(|u| u.player_id != game.my_id)
                                .filter(|u| {
                                    let line = Line::new(me.position, u.position);
                                    !self
                                        .constants
                                        .obstacles
                                        .iter()
                                        .filter(|o| !o.can_shoot_through)
                                        .any(|o| {
                                            let circle = o.as_circle(-self.constants.unit_radius);
                                            circle.intercept_with_line(&line)
                                        })
                                })
                                .min_by(|a, b| {
                                    let a_value = a.position.square_distance(&me.position);
                                    let b_value = b.position.square_distance(&me.position);
                                    a_value.partial_cmp(&b_value).unwrap()
                                })
                        } else {
                            None
                        };

                    let my_weapon_range = me
                        .weapon
                        .map(|w| {
                            let prop = &self.constants.weapons[w as usize];
                            prop.projectile_speed * prop.projectile_life_time
                        })
                        .unwrap_or(0.0);

                    let target_direction = if closest_enemy.is_some()
                        && closest_enemy
                            .unwrap()
                            .position
                            .square_distance(&me.position)
                            <= my_weapon_range.powi(2) * 1.5
                    {
                        let distance = closest_enemy.unwrap().position.distance(&me.position)
                            - self.constants.unit_radius;
                        let seconds_to_enemy = distance
                            / self.constants.weapons[me.weapon.unwrap() as usize].projectile_speed;
                        closest_enemy.unwrap().position
                            + closest_enemy.unwrap().velocity * seconds_to_enemy * 0.77
                            - me.position
                    } else {
                        if let Some(sound) = self.pp2.sounds().iter().min_by(|a, b| {
                            let a_dist = me.position.square_distance(&a.position);
                            let b_dist = me.position.square_distance(&b.position);
                            a_dist.partial_cmp(&b_dist).unwrap()
                        }) {
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
                            action: if closest_enemy.is_some()
                                && closest_enemy
                                    .unwrap()
                                    .position
                                    .square_distance(&me.position)
                                    < 8_000.0
                            {
                                let closest_enemy = closest_enemy.unwrap();
                                let weapon_id = me.weapon.unwrap() as usize;
                                let weapon = &self.constants.weapons[weapon_id];
                                let weapon_range = weapon.projectile_life_time
                                    * weapon.projectile_speed
                                    + self.constants.unit_radius * 2.0;
                                let mut aim = Line::new(
                                    me.position,
                                    me.position + (me.direction.normalize() * weapon_range),
                                );
                                let d = closest_enemy.position.distance(&me.position);
                                if aim.length() > d + self.constants.unit_radius * 2.0 {
                                    aim.set_length(d + self.constants.unit_radius * 2.0)
                                }

                                let seconds_to_enemy = (d - self.constants.unit_radius)
                                    / self.constants.weapons[me.weapon.unwrap() as usize]
                                        .projectile_speed;

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
                                    closest_enemy.position
                                        + closest_enemy.velocity * seconds_to_enemy * 0.77,
                                    self.constants.unit_radius * 0.9,
                                );
                                let obstacle_on_line = self
                                    .constants
                                    .obstacles
                                    .iter()
                                    .filter(|o| !o.can_shoot_through)
                                    .any(|o| o.as_circle(0.0).intercept_with_line(&aim));

                                if obstacle_on_line && d > weapon_range {
                                    if let Some(loot) = game
                                        .loot
                                        .iter()
                                        .filter(|l| {
                                            l.position.distance(&me.position)
                                                <= self.constants.unit_radius
                                                && l.is_useful_to_me(me, &self.constants)
                                        })
                                        .next()
                                    {
                                        Some(ActionOrder::Pickup { loot: loot.id })
                                    } else if me.shield < self.constants.max_shield
                                        && me.shield_potions > 0
                                    {
                                        Some(ActionOrder::UseShieldPotion {})
                                    } else {
                                        None
                                    }
                                } else {
                                    Some(ActionOrder::Aim {
                                        shoot: !obstacle_on_line
                                            && enemy_circle.intercept_with_line(&aim)
                                            && d <= weapon_range,
                                    })
                                }
                            } else if let Some(loot) = game
                                .loot
                                .iter()
                                .filter(|l| {
                                    l.position.distance(&me.position) <= self.constants.unit_radius
                                        && l.is_useful_to_me(me, &self.constants)
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
