use crate::debug_interface::DebugInterface;
use ai_cup_22::model::*;
use ai_cup_22::potential_field::*;
use std::f64::consts::PI;

pub struct MyStrategy {
    constants: Constants,
    pp: PotentialField,
    obstacles_cant_shoot_through: Vec<Obstacle>,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        let pp = PotentialField::new(&constants);
        Self {
            obstacles_cant_shoot_through: constants
                .obstacles
                .iter()
                .filter(|o| !o.can_shoot_through)
                .cloned()
                .collect(),
            constants,
            pp,
        }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        // let debug_interface = _debug_interface.unwrap();
        self.pp.update(game);

        let mut enemies: Vec<&Unit> = game
            .units
            .iter()
            .filter(|u| u.player_id != game.my_id)
            .collect();
        enemies.extend(self.pp.old_enemies.iter());

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
                    // debug_interface.add_arc(
                    //     me.position,
                    //     self.constants.view_distance,
                    //     0.5,
                    //     normalize_angle(me.direction.angle() - me.fov_angle(&self.constants) / 2.0),
                    //     normalize_angle(me.direction.angle() + me.fov_angle(&self.constants) / 2.0),
                    //     Color::new(0.0, 0.0, 0.0, 0.8),
                    // );
                    // debug_interface.add_segment(
                    //     me.position,
                    //     me.position
                    //         + me.direction
                    //             .rotate(-me.fov_angle(&self.constants) / 2.0)
                    //             .normalize()
                    //             * self.constants.view_distance,
                    //     0.5,
                    //     Color::new(0.0, 0.0, 0.0, 0.8),
                    // );
                    // debug_interface.add_segment(
                    //     me.position,
                    //     me.position
                    //         + me.direction
                    //             .rotate(me.fov_angle(&self.constants) / 2.0)
                    //             .normalize()
                    //             * self.constants.view_distance,
                    //     0.5,
                    //     Color::new(0.0, 0.0, 0.0, 0.8),
                    // );
                    // for enemy in self.pp.old_enemies.iter() {
                    //     if me.is_in_fov(enemy.position, &self.constants) {
                    //         debug_interface.add_circle(
                    //             enemy.position,
                    //             1.0,
                    //             Color::new(1.0, 0.0, 0.0, 0.5),
                    //         );
                    //     } else {
                    //         debug_interface.add_circle(
                    //             enemy.position,
                    //             1.0,
                    //             Color::new(0.0, 1.0, 0.0, 0.5),
                    //         );
                    //     }
                    // }
                    let i_have_weapon =
                        me.weapon.is_some() && me.ammo[me.weapon.unwrap() as usize] > 0;

                    let enemy_has_good_weapon = enemies.iter().any(|enemy| {
                        if let Some(weapon) = enemy.weapon {
                            (weapon == BOW || weapon == STAFF) && enemy.ammo[weapon as usize] > 0
                        } else {
                            false
                        }
                    });
                    let me_has_good_weapon = if let Some(weapon) = me.weapon {
                        (weapon == BOW || weapon == STAFF) && me.ammo[weapon as usize] > 0
                    } else {
                        false
                    };

                    let fight_mode = if i_have_weapon {
                        if me_has_good_weapon || !enemy_has_good_weapon {
                            FightMode::Attack
                        } else {
                            FightMode::Defend
                        }
                    } else {
                        FightMode::RunWithNoWeapons
                    };

                    // match fight_mode {
                    //     FightMode::Attack => debug_interface.add_placed_text(
                    //         me.position + Vec2::new(0.0, 1.0),
                    //         "Attack".to_string(),
                    //         Vec2::zero(),
                    //         0.5,
                    //         Color::BLACK,
                    //     ),
                    //     FightMode::Defend => debug_interface.add_placed_text(
                    //         me.position + Vec2::new(0.0, 1.0),
                    //         "Defend".to_string(),
                    //         Vec2::zero(),
                    //         0.5,
                    //         Color::BLACK,
                    //     ),
                    //     FightMode::RunWithNoWeapons => debug_interface.add_placed_text(
                    //         me.position + Vec2::new(0.0, 1.0),
                    //         "Run".to_string(),
                    //         Vec2::zero(),
                    //         0.5,
                    //         Color::BLACK,
                    //     ),
                    // }

                    // for point in self.pp.points_around(me.id).iter() {
                    //     let value = self.pp.value(*point, me, fight_mode);
                    //     // debug_interface.add_circle(*point, 0.5, color_by_value(value));
                    //     debug_interface.add_placed_text(
                    //         *point,
                    //         format!("{:.2}", value),
                    //         Vec2::new(0.5, 0.5),
                    //         0.2,
                    //         Color::BLACK,
                    //     );
                    // }

                    let my_weapon_range = me
                        .weapon
                        .map(|w| {
                            let prop = &self.constants.weapons[w as usize];
                            prop.projectile_speed * prop.projectile_life_time
                        })
                        .unwrap_or(0.0);

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
                                    && !self.obstacles_cant_shoot_through.iter().any(|o| {
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
                                    && self
                                        .obstacles_cant_shoot_through
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
                    let is_in_danger = self.pp.is_in_danger(me)
                        || (closest_enemy.is_some()
                            && closest_enemy.unwrap().position.distance_to(&me.position)
                                < my_weapon_range);

                    let target_velocity = if me.remaining_spawn_time.is_some() {
                        (*self
                            .pp
                            .points_around(me.id)
                            .iter()
                            .max_by(|&a, &b| {
                                let a_value = self.pp.value_unspawned(*a, me);
                                let b_value = self.pp.value_unspawned(*b, me);
                                a_value.partial_cmp(&b_value).unwrap()
                            })
                            .unwrap_or(&game.zone.current_center)
                            - me.position)
                            .normalize()
                            * self.constants.spawn_movement_speed
                    } else if is_in_danger {
                        let best_pp =
                            self.pp
                                .points_around(me.id)
                                .iter()
                                .cloned()
                                .max_by(|&a, &b| {
                                    let a_value = self.pp.value(a, me, fight_mode);
                                    let b_value = self.pp.value(b, me, fight_mode);
                                    a_value.partial_cmp(&b_value).unwrap()
                                });
                        (best_pp.unwrap_or(game.zone.current_center) - me.position).normalize()
                            * self.constants.max_unit_forward_speed
                    } else {
                        let bonus = self
                            .pp
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
                                        > self.constants.unit_radius * 8.0
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
                                let vec = if me.position.distance_to(&game.zone.current_center)
                                    < 0.5 * game.zone.current_radius
                                {
                                    (me.position - game.zone.current_center) * 1.5
                                } else {
                                    (me.position - game.zone.current_center).rotate(PI / 10.0)
                                };
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

                                        (seconds_to_unit + weapon.aim_time) > respawning_time
                                            && u.as_circle(self.constants.unit_radius)
                                                .intercept_with_line(&aim)
                                    });

                                if obstacles_on_line > 1 || d > weapon_range || unit_on_line {
                                    if let Some(loot) = self
                                        .pp
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
                                    } else if me.shield
                                        <= (self.constants.max_shield
                                            - self.constants.shield_per_potion)
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
                                            && d <= (weapon_range
                                                + self.constants.unit_radius * 2.0)
                                            && remaining_spawn_time < seconds_to_unspawned_enemy,
                                    })
                                }
                            } else if let Some(loot) = self
                                .pp
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
                            } else if me.shield
                                <= (self.constants.max_shield - self.constants.shield_per_potion)
                                && me.shield_potions > 0
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
