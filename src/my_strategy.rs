use crate::debug_interface::DebugInterface;
// use ai_cup_22::debugging::Color;
use ai_cup_22::model::*;
use ai_cup_22::potential_field::{FightMode, PotentialField};
use std::f64::consts::PI;

pub struct MyStrategy {
    constants: Constants,
    pp: PotentialField,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        let pp = PotentialField::new(&constants);
        Self { constants, pp }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        // let debug_interface = _debug_interface.unwrap();
        self.pp.update(game);

        let spawned_unit = game
            .units
            .iter()
            .find(|u| u.player_id == game.my_id && u.remaining_spawn_time.is_none());

        let obstacles: Vec<&Obstacle> = self
            .constants
            .obstacles
            .iter()
            .filter(|o| !o.can_shoot_through)
            .collect();
        let enemies: Vec<&Unit> = game
            .units
            .iter()
            .filter(|u| u.player_id != game.my_id)
            .collect();

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
                    //     // debug_interface.add_circle(*point, 0.5, color_by_value(value));
                    //     debug_interface.add_placed_text(
                    //         *point,
                    //         format!("{:.2}", value),
                    //         Vec2::new(0.5, 0.5),
                    //         0.2,
                    //         Color::BLACK,
                    //     );
                    // }
                    let i_have_weapon =
                        me.weapon.is_some() && me.ammo[me.weapon.unwrap() as usize] > 0;

                    let my_weapon_range = me
                        .weapon
                        .map(|w| {
                            let prop = &self.constants.weapons[w as usize];
                            prop.projectile_speed * prop.projectile_life_time
                        })
                        .unwrap_or(0.0);

                    // TODO: запоминать врагов
                    let closest_enemy = if i_have_weapon {
                        let can_shoot_right_now = enemies
                            .iter()
                            .filter(|u| {
                                let distance = u.position.distance_to(&me.position)
                                    - self.constants.unit_radius;
                                let seconds_to_enemy = distance
                                    / self.constants.weapons[me.weapon.unwrap() as usize]
                                        .projectile_speed;

                                let line = Line::new(me.position, u.position);
                                u.remaining_spawn_time.unwrap_or(0.0) < seconds_to_enemy
                                    && !obstacles.iter().any(|o| {
                                        let circle = o.as_circle(-self.constants.unit_radius);
                                        circle.intercept_with_line(&line)
                                    })
                                    && !game
                                        .units
                                        .iter()
                                        .filter(|u| u.player_id == game.my_id && u.id != me.id)
                                        .any(|u| {
                                            let circle = u.as_circle(self.constants.unit_radius);
                                            circle.intercept_with_line(&line)
                                        })
                            })
                            .min_by(|a, b| {
                                let a_value = a.position.square_distance_to(&me.position);
                                let b_value = b.position.square_distance_to(&me.position);
                                a_value.partial_cmp(&b_value).unwrap()
                            });
                        let can_shoot_probably = enemies
                            .iter()
                            .filter(|u| {
                                let distance = u.position.distance_to(&me.position)
                                    - self.constants.unit_radius;
                                let seconds_to_enemy = distance
                                    / self.constants.weapons[me.weapon.unwrap() as usize]
                                        .projectile_speed;

                                let line = Line::new(me.position, u.position);
                                u.remaining_spawn_time.unwrap_or(0.0) < seconds_to_enemy
                                    && obstacles
                                        .iter()
                                        .filter(|o| {
                                            let circle = o.as_circle(-self.constants.unit_radius);
                                            circle.intercept_with_line(&line)
                                        })
                                        .count()
                                        < 2
                                    && !game
                                        .units
                                        .iter()
                                        .filter(|u| u.player_id == game.my_id && u.id != me.id)
                                        .any(|u| {
                                            let circle = u.as_circle(self.constants.unit_radius);
                                            circle.intercept_with_line(&line)
                                        })
                            })
                            .min_by(|a, b| {
                                let a_value = a.position.square_distance_to(&me.position);
                                let b_value = b.position.square_distance_to(&me.position);
                                a_value.partial_cmp(&b_value).unwrap()
                            });

                        if let Some(enemy) = can_shoot_right_now {
                            Some(enemy)
                        } else if let Some(enemy) = can_shoot_probably {
                            Some(enemy)
                        } else {
                            enemies.iter().min_by(|a, b| {
                                let a_value = a.position.square_distance_to(&me.position);
                                let b_value = b.position.square_distance_to(&me.position);
                                a_value.partial_cmp(&b_value).unwrap()
                            })
                        }
                    } else {
                        None
                    };

                    // TODO: get targets from other units
                    let is_very_danger = self.pp.is_in_very_danger(me);

                    let enemies_sum_hp = enemies.iter().map(|u| u.health + u.shield).sum::<f64>();

                    let fight_mode = if i_have_weapon {
                        if enemies_sum_hp < (me.health + me.shield) {
                            FightMode::Attack
                        } else {
                            FightMode::Defend
                        }
                    } else {
                        FightMode::RunWithNoWeapons
                    };

                    let target_velocity = if !i_have_weapon && !is_very_danger {
                        let target_position = if let Some(weapon_idx) = me.weapon {
                            game.loot.iter().find(|l| l.is_ammo_for(weapon_idx))
                        } else {
                            game.loot.iter().find(|l| l.is_weapon())
                        };
                        let target_position = target_position
                            .map(|l| l.position)
                            .unwrap_or_else(|| me.position + me.direction.normalize() * 100.0);

                        (target_position - me.position).normalize()
                            * self.constants.max_unit_forward_speed
                    } else if me.remaining_spawn_time.is_some()
                        && !self.pp.im_inside_obstacle(me)
                        && !self.pp.im_outside(me)
                    {
                        if let Some(spawned_unit) = spawned_unit {
                            let distance = spawned_unit.position.distance_to(&me.position);
                            (spawned_unit.position - me.position).normalize()
                                * (distance - self.constants.unit_radius * 4.0)
                        } else {
                            let nearest_unspawned_unit = game
                                .units
                                .iter()
                                .filter(|u| {
                                    u.player_id == game.my_id
                                        && u.remaining_spawn_time.is_some()
                                        && u.id != me.id
                                })
                                .min_by(|a, b| {
                                    a.position
                                        .square_distance_to(&me.position)
                                        .partial_cmp(&b.position.square_distance_to(&me.position))
                                        .unwrap()
                                });
                            if let Some(nearest_unspawned_unit) = nearest_unspawned_unit {
                                let distance =
                                    nearest_unspawned_unit.position.distance_to(&me.position);
                                if distance > self.constants.unit_radius * 7.0 {
                                    (nearest_unspawned_unit.position - me.position).normalize()
                                        * self.constants.spawn_movement_speed
                                } else if distance < self.constants.unit_radius * 3.0 {
                                    (nearest_unspawned_unit.position - me.position)
                                        .normalize()
                                        .inverse()
                                        * self.constants.spawn_movement_speed
                                } else {
                                    if let Some(loot) = game
                                        .loot
                                        .iter()
                                        .filter(|l| l.is_first_take_loot())
                                        .min_by(|a, b| {
                                            a.position
                                                .square_distance_to(&me.position)
                                                .partial_cmp(
                                                    &b.position.square_distance_to(&me.position),
                                                )
                                                .unwrap()
                                        })
                                    {
                                        (loot.position - me.position).normalize()
                                            * self.constants.spawn_movement_speed
                                    } else {
                                        Vec2::new(0.0, 0.0)
                                    }
                                }
                            } else {
                                if let Some(loot) =
                                    game.loot.iter().filter(|l| l.is_first_take_loot()).min_by(
                                        |a, b| {
                                            a.position
                                                .square_distance_to(&me.position)
                                                .partial_cmp(
                                                    &b.position.square_distance_to(&me.position),
                                                )
                                                .unwrap()
                                        },
                                    )
                                {
                                    (loot.position - me.position).normalize()
                                        * self.constants.spawn_movement_speed
                                } else {
                                    Vec2::new(0.0, 0.0)
                                }
                            }
                        }
                    } else if self.pp.is_in_danger(me) {
                        (*self
                            .pp
                            .points_around(me.id)
                            .iter()
                            .max_by(|a, b| {
                                let a_value = self.pp.value(a, me, fight_mode);
                                let b_value = self.pp.value(b, me, fight_mode);
                                a_value.partial_cmp(&b_value).unwrap()
                            })
                            .unwrap_or(&game.zone.current_center)
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
                                    && l.position.distance_to(&game.zone.current_center)
                                        < game.zone.current_radius
                                            - self.constants.unit_radius * 2.0
                            })
                            .min_by(|l, r| {
                                l.position
                                    .square_distance_to(&me.position)
                                    .partial_cmp(&r.position.square_distance_to(&me.position))
                                    .unwrap()
                            });

                        let target_position = if let Some(bonus) = bonus {
                            bonus.position
                        } else {
                            let nearest_ally = game
                                .units
                                .iter()
                                .filter(|u| u.player_id == game.my_id && u.id != me.id)
                                .filter(|u| {
                                    u.position.distance_to(&me.position)
                                        > self.constants.unit_radius * 4.0
                                })
                                .min_by(|a, b| {
                                    a.position
                                        .square_distance_to(&me.position)
                                        .partial_cmp(&b.position.square_distance_to(&me.position))
                                        .unwrap()
                                });
                            if let Some(ally) = nearest_ally {
                                ally.position
                            } else {
                                let vec =
                                    (me.position - game.zone.current_center).rotate(PI / 10.0);
                                game.zone.current_center + vec
                            }
                        };

                        (target_position - me.position).normalize()
                            * self.constants.max_unit_forward_speed
                    };

                    let target_direction = if closest_enemy.is_some()
                        && closest_enemy
                            .unwrap()
                            .position
                            .square_distance_to(&me.position)
                            <= my_weapon_range.powi(2) * 1.5
                    {
                        let distance = closest_enemy.unwrap().position.distance_to(&me.position)
                            - self.constants.unit_radius;
                        let seconds_to_enemy = distance
                            / self.constants.weapons[me.weapon.unwrap() as usize].projectile_speed;
                        closest_enemy.unwrap().position
                            + closest_enemy.unwrap().velocity * seconds_to_enemy * 0.77
                            - me.position
                    } else if let Some(sound) = self.pp.sounds().iter().min_by(|a, b| {
                        let a_dist = me.position.square_distance_to(&a.position);
                        let b_dist = me.position.square_distance_to(&b.position);
                        a_dist.partial_cmp(&b_dist).unwrap()
                    }) {
                        sound.position - me.position
                    } else {
                        target_velocity
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
                                    .square_distance_to(&me.position)
                                    < my_weapon_range.powi(2) * 1.5
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
                                let d = closest_enemy.position.distance_to(&me.position);
                                if aim.length() > d + self.constants.unit_radius * 2.0 {
                                    aim.set_length(d + self.constants.unit_radius * 2.0)
                                }

                                let seconds_to_enemy =
                                    (d + self.constants.unit_radius) / weapon.projectile_speed;

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
                                let obstacles_on_line = self
                                    .constants
                                    .obstacles
                                    .iter()
                                    .filter(|o| !o.can_shoot_through)
                                    .filter(|o| o.as_circle(0.0).intercept_with_line(&aim))
                                    .count();
                                let unit_on_line = game
                                    .units
                                    .iter()
                                    .filter(|u| u.player_id == game.my_id && u.id != me.id)
                                    .any(|u| {
                                        let respawning_time = u.remaining_spawn_time.unwrap_or(0.0);
                                        let d = u.position.distance_to(&me.position);
                                        let seconds_to_unit = (d + self.constants.unit_radius)
                                            / weapon.projectile_speed;

                                        seconds_to_unit > respawning_time
                                            && u.as_circle(self.constants.unit_radius)
                                                .intercept_with_line(&aim)
                                    });

                                if obstacles_on_line > 1 || d > weapon_range {
                                    if let Some(loot) = game
                                        .loot
                                        .iter()
                                        .filter(|l| {
                                            l.position.distance_to(&me.position)
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
                                    let remaining_spawn_time =
                                        closest_enemy.remaining_spawn_time.unwrap_or(-1.0);
                                    let seconds_to_unspawned_enemy =
                                        (d - self.constants.unit_radius) / weapon.projectile_speed;
                                    Some(ActionOrder::Aim {
                                        shoot: obstacles_on_line == 0
                                            && !unit_on_line
                                            && enemy_circle.intercept_with_line(&aim)
                                            && d <= weapon_range + self.constants.unit_radius * 2.0
                                            && remaining_spawn_time < seconds_to_unspawned_enemy,
                                    })
                                }
                            } else if let Some(loot) = game
                                .loot
                                .iter()
                                .filter(|l| {
                                    l.position.distance_to(&me.position)
                                        <= self.constants.unit_radius
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
