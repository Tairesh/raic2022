use crate::model::*;
use crate::potential_field::FightMode;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::ops::Not;

pub fn normalize_angle(angle: f64) -> f64 {
    let mut angle = angle;
    while angle < -PI {
        angle += 2.0 * PI;
    }
    while angle > PI {
        angle -= 2.0 * PI;
    }
    angle
}

pub struct PotentialField {
    constants: Constants,
    seeing_units: Vec<Unit>,
    pub old_enemies: Vec<Unit>,
    seeing_projectiles: Vec<Projectile>,
    pub dangerous_projectiles: Vec<Projectile>,
    pub old_projectiles: Vec<Projectile>,
    zone: Zone,
    pub shooting_sounds: Vec<(Sound, Vec2, i32)>,
    pub hit_sounds: Vec<(Sound, i32)>,
    pub steps_sounds: Vec<(Sound, i32)>,
    pub loot: Vec<Loot>,
    current_tick: i32,
    my_id: i32,
}

impl PotentialField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            constants: constants.clone(),
            seeing_units: Vec::new(),
            old_enemies: Vec::new(),
            seeing_projectiles: Vec::new(),
            old_projectiles: Vec::new(),
            dangerous_projectiles: Vec::new(),
            zone: Zone::default(),
            shooting_sounds: Vec::new(),
            hit_sounds: Vec::new(),
            steps_sounds: Vec::new(),
            loot: Vec::new(),
            current_tick: 0,
            my_id: 0,
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.current_tick = game.current_tick;
        self.my_id = game.my_id;
        self.old_enemies.extend(self.seeing_units.iter().cloned());
        self.seeing_units = game.units.clone();
        let seeing_units_ids = self
            .seeing_units
            .iter()
            .map(|u| u.id)
            .collect::<HashSet<_>>();
        self.old_enemies.retain(|e| {
            !seeing_units_ids.contains(&e.id)
                && !game
                    .units
                    .iter()
                    .filter(|u| u.player_id == game.my_id)
                    .any(|u| {
                        u.is_in_fov(e.position, &self.constants)
                            && !game
                                .units
                                .iter()
                                .filter(|u| u.player_id == game.my_id)
                                .all(|u| {
                                    u.position.distance_to(&e.position)
                                        > self.constants.view_distance * 1.1
                                })
                    })
        });
        self.old_enemies.iter_mut().for_each(|enemy| {
            let closest_my_unit = game
                .units
                .iter()
                .filter(|u| u.player_id == game.my_id)
                .min_by(|a, b| {
                    a.position
                        .square_distance_to(&enemy.position)
                        .partial_cmp(&b.position.square_distance_to(&enemy.position))
                        .unwrap()
                });
            if let Some(closest_my_unit) = closest_my_unit {
                enemy.direction = (closest_my_unit.position - enemy.position).normalize();
            }
            enemy.weapon = Some(BOW);
            enemy.ammo[BOW as usize] = 25;
            if let Some(remaining_spawn_time) = enemy.remaining_spawn_time {
                enemy.remaining_spawn_time = if remaining_spawn_time >= 0.0 {
                    Some(remaining_spawn_time - 1.0 / self.constants.ticks_per_second)
                } else {
                    None
                };
            }
        });
        self.seeing_projectiles = game.projectiles.clone();
        self.zone = game.zone.clone();
        self.old_projectiles
            .iter_mut()
            .for_each(|p| p.life_time -= 1.0 / self.constants.ticks_per_second);
        let seeing_projectiles_ids = self
            .seeing_projectiles
            .iter()
            .map(|p| p.id)
            .collect::<HashSet<i32>>();
        self.old_projectiles
            .retain(|p| p.life_time >= 0.0 && !seeing_projectiles_ids.contains(&p.id));

        self.dangerous_projectiles = game
            .projectiles
            .iter()
            .filter(|projectile| {
                game.units
                    .iter()
                    .filter(|u| u.player_id == game.my_id)
                    .any(|me| projectile.is_dangerous(me, &self.constants))
            })
            .cloned()
            .collect();
        self.dangerous_projectiles.extend(
            self.old_projectiles
                .iter()
                .filter(|projectile| {
                    game.units
                        .iter()
                        .filter(|u| u.player_id == game.my_id)
                        .any(|me| projectile.is_dangerous(me, &self.constants))
                })
                .cloned(),
        );

        self.old_projectiles
            .extend(self.seeing_projectiles.iter().cloned());

        self.shooting_sounds.extend(
            game.sounds
                .iter()
                .filter(|sound| {
                    let name = &self.constants.sounds[sound.type_index as usize].name;
                    match name.as_str() {
                        "Wand" | "Staff" | "Bow" => true,
                        _ => false,
                    }
                })
                .filter(|sound| {
                    !game
                        .units
                        .iter()
                        .filter(|unit| unit.player_id != game.my_id)
                        .any(|unit| {
                            let distance = unit.position.distance_to(&sound.position);
                            let props = &self.constants.sounds[sound.type_index as usize];
                            distance <= self.constants.unit_radius + props.offset
                        })
                })
                .map(|sound| {
                    let unit = game
                        .units
                        .iter()
                        .find(|unit| unit.id == sound.unit_id)
                        .unwrap();
                    (sound.clone(), unit.position, game.current_tick)
                }),
        );
        self.shooting_sounds
            .retain(|&(.., tick)| game.current_tick - tick < 50);
        self.hit_sounds.extend(
            game.sounds
                .iter()
                .filter(|sound| {
                    let name = &self.constants.sounds[sound.type_index as usize].name;
                    match name.as_str() {
                        "WandHit" | "StaffHit" | "BowHit" => true,
                        _ => false,
                    }
                })
                .filter(|sound| {
                    !game
                        .units
                        .iter()
                        .filter(|unit| unit.player_id != game.my_id)
                        .any(|unit| {
                            let distance = unit.position.distance_to(&sound.position);
                            let props = &self.constants.sounds[sound.type_index as usize];
                            distance <= self.constants.unit_radius + props.offset
                        })
                })
                .map(|sound| (sound.clone(), game.current_tick)),
        );
        self.hit_sounds
            .retain(|&(.., tick)| game.current_tick - tick < 50);
        self.steps_sounds.extend(
            game.sounds
                .iter()
                .filter(|sound| {
                    let name = &self.constants.sounds[sound.type_index as usize].name;
                    match name.as_str() {
                        "Steps" => true,
                        _ => false,
                    }
                })
                .filter(|sound| {
                    !game
                        .units
                        .iter()
                        .filter(|unit| unit.player_id != game.my_id)
                        .any(|unit| {
                            let distance = unit.position.distance_to(&sound.position);
                            let props = &self.constants.sounds[sound.type_index as usize];
                            distance <= self.constants.unit_radius + props.offset
                        })
                })
                .map(|sound| (sound.clone(), game.current_tick)),
        );
        self.steps_sounds
            .retain(|&(.., tick)| game.current_tick - tick < 50);

        self.loot.retain(|loot| {
            !game
                .units
                .iter()
                .filter(|p| p.player_id == game.my_id)
                .any(|p| p.is_in_fov(loot.position, &self.constants))
        });
        self.loot.extend(game.loot.iter().cloned());
    }

    pub fn points_around(&self, unit_id: i32) -> Vec<Vec2> {
        let me = self.seeing_units.iter().find(|u| u.id == unit_id).unwrap();
        // where unit will be in 3 ticks
        let inertion = (me.velocity / self.constants.ticks_per_second) * 3.0;
        let position = me.position + inertion;

        (0..8)
            .map(|i| {
                // TODO: может быть нужно больше точек?
                let angle = (i * 45) as f64 * PI / 180.0;
                Vec2::new(
                    position.x + angle.cos() * self.constants.unit_radius,
                    position.y + angle.sin() * self.constants.unit_radius,
                )
            })
            .filter(|p| {
                self.constants
                    .obstacles
                    .iter()
                    .any(|o| o.as_circle(self.constants.unit_radius).contains(p))
                    .not()
                    && self
                        .seeing_units
                        .iter()
                        .filter(|u| u.id != unit_id)
                        .any(|u| u.position.distance_to(p) <= self.constants.unit_radius * 2.0)
                        .not()
            })
            .collect()
    }

    fn value_projectiles(&self, position: Vec2) -> f64 {
        let mut value = 0.0;
        for projectile in self.dangerous_projectiles.iter() {
            let line = projectile.as_line();

            let distance = line.distance_to_point(&position);
            let distance_limit = self.constants.unit_radius * 5.0;

            if distance > distance_limit {
                continue;
            }

            value -= 1.0 - distance / distance_limit;
            let distance = projectile.position.distance_to(&position);
            let range = projectile.range();

            value += 0.05 * distance / range;
        }
        value
    }

    fn value_zone(&self, position: Vec2) -> f64 {
        let distance = self.zone.next_center.distance_to(&position);
        let wanna_radius = (self.zone.next_radius - self.constants.unit_radius * 4.0)
            .max(self.constants.unit_radius * 2.0);
        if distance < wanna_radius {
            return distance / wanna_radius;
        }

        1.0 - distance / wanna_radius
    }

    fn value_outside(&self, position: Vec2) -> f64 {
        let distance = self.zone.current_center.distance_to(&position);
        let max_distance = self.zone.current_radius - self.constants.unit_radius * 4.0;
        if distance > max_distance {
            return -(distance - max_distance);
        }

        0.0
    }

    fn value_shooting_sounds(&self, position: Vec2) -> f64 {
        let mut value = 0.0;
        for (sound, my_pos, tick) in self.shooting_sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;

            let line = Line::new(sound.position, *my_pos);
            let distance = line.distance_to_point(&position);
            let distance_limit = self.constants.unit_radius * 4.0;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * tick_k;
        }
        value
    }

    fn value_hit_sounds(&self, position: Vec2) -> f64 {
        let mut value = 0.0;
        for (sound, tick) in self.hit_sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;
            let distance = sound.position.distance_to(&position);
            let distance_limit = self.constants.unit_radius * 5.0;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * tick_k;
        }
        value
    }

    fn value_steps_sounds(&self, position: Vec2) -> f64 {
        let mut value = 0.0;
        for (sound, tick) in self.steps_sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;
            let distance = sound.position.distance_to(&position);
            let distance_limit = self.constants.unit_radius * 5.0;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * tick_k;
        }
        value
    }

    fn enemy_val(&self, position: Vec2, me: &Unit, fight_mode: FightMode, enemy: &Unit) -> f64 {
        let mut value = 0.0;
        let my_range = me.range(&self.constants);
        let distance_to_enemy = enemy.position.distance_to(&position);
        let enemy_range = enemy.range(&self.constants);
        match fight_mode {
            FightMode::Attack => {
                // отходим на границу моего ренджа, подходим ближе если не можем стрелять, не ближе 20.0 примерно (уверенного попадания из лука и огнемета), нападаем если у него нет оружия
                let target_distance = if let Some(my_range) = my_range {
                    my_range * 0.75
                } else {
                    40.0
                };

                if distance_to_enemy < target_distance {
                    value -= 1.0 - distance_to_enemy / target_distance;
                } else if distance_to_enemy < target_distance * 4.0 {
                    value += (distance_to_enemy / target_distance - 1.0) * 0.5;
                }
            }
            FightMode::Defend => {
                // отходим за границу его ренджа, на наш не особо обращаем внимание, но нападаем если у него нет оружия
                if let Some(enemy_range) = enemy_range {
                    value -= 1.0 - distance_to_enemy / enemy_range;
                } else {
                    let target_distance = my_range.unwrap_or(40.0) * 0.75;
                    if distance_to_enemy < target_distance {
                        value -= 1.0 - distance_to_enemy / target_distance;
                    } else if distance_to_enemy < target_distance * 2.0 {
                        value += (distance_to_enemy / target_distance - 1.0) * 0.25;
                    }
                }
            }
            FightMode::RunWithNoWeapons => {
                // убегаем за границу видимости
                let view_distance = self.constants.view_distance + self.constants.unit_radius * 2.0;
                if distance_to_enemy < view_distance {
                    value -= 1.0 - distance_to_enemy / view_distance;
                }
            }
        }

        if let Some(enemy_range) = enemy_range {
            let aim = Line::new(
                enemy.position,
                enemy.position + enemy.direction.normalize() * enemy_range,
            );

            let distance = aim.distance_to_point(&position);
            let distance_limit = self.constants.unit_radius * 3.0;
            if distance > distance_limit {
                return value;
            }

            value -= (1.0 - distance / distance_limit) * 0.5;
        }

        value
    }

    fn value_enemies(&self, position: Vec2, me: &Unit, fight_mode: FightMode) -> f64 {
        let mut value = 0.0;

        for enemy in self
            .seeing_units
            .iter()
            .filter(|u| u.player_id != self.my_id)
        {
            value += self.enemy_val(position, me, fight_mode, enemy);
        }

        for enemy in self.old_enemies.iter() {
            value += self.enemy_val(position, me, fight_mode, enemy);
        }

        value
    }

    pub fn value_allies(&self, position: Vec2, me: &Unit) -> f64 {
        let mut value = 0.0;

        for ally in self
            .seeing_units
            .iter()
            .filter(|u| u.player_id == self.my_id && u.id != me.id)
        {
            let distance_to_ally = ally.position.distance_to(&position);
            let min_distance = self.constants.unit_radius * 3.0;
            let max_distance = self.constants.unit_radius * 10.0;
            if distance_to_ally < min_distance {
                value -= 1.0 - distance_to_ally / min_distance;
            } else if distance_to_ally > max_distance {
                value += max_distance / distance_to_ally
            }
        }

        value
    }

    pub fn value_loot(&self, position: Vec2, me: &Unit) -> f64 {
        let mut value = 0.0;

        for loot in self
            .loot
            .iter()
            .filter(|l| l.is_useful_to_me(me, &self.constants))
        {
            let distance_to_loot = loot.position.distance_to(&position);
            let max_distance = self.constants.unit_radius * 10.0;
            if distance_to_loot < max_distance {
                value += 1.0 - distance_to_loot / max_distance;
            }
        }

        value
    }

    pub fn im_inside_obstacle(&self, me: &Unit) -> bool {
        self.constants.obstacles.iter().any(|o| {
            o.as_circle(self.constants.unit_radius)
                .contains(&me.position)
        })
    }

    pub fn im_outside(&self, me: &Unit) -> bool {
        me.position.distance_to(&self.zone.current_center)
            >= (self.zone.current_radius - self.constants.unit_radius * 4.0)
    }

    pub fn is_in_danger(&self, me: &Unit) -> bool {
        self.dangerous_projectiles
            .iter()
            .any(|p| p.is_dangerous(me, &self.constants))
            || self.shooting_sounds.iter().any(|(s, _, _)| {
                if me.is_in_fov(s.position, &self.constants) {
                    return false;
                }
                let range = s.get_weapon_shooting_range(&self.constants);
                let offset = self.constants.sounds[s.type_index as usize].offset;
                s.position.distance_to(&me.position) <= (range + offset)
            })
            || self.hit_sounds.iter().any(|(s, _)| {
                let offset = self.constants.sounds[s.type_index as usize].offset;
                s.position.distance_to(&me.position) <= (self.constants.unit_radius + offset)
            })
            || me.position.distance_to(&self.zone.current_center)
                >= (self.zone.current_radius - self.constants.unit_radius * 4.0)
    }

    pub fn sounds(&self) -> Vec<Sound> {
        let mut sounds = self
            .shooting_sounds
            .iter()
            .map(|(sound, ..)| sound.clone())
            .collect::<Vec<Sound>>();

        sounds.extend(self.hit_sounds.iter().map(|(sound, _)| sound.clone()));
        sounds.extend(self.steps_sounds.iter().map(|(sound, _)| sound.clone()));

        sounds
    }

    pub fn value(&self, position: Vec2, me: &Unit, fight_mode: FightMode) -> f64 {
        if self
            .dangerous_projectiles
            .iter()
            .any(|p| p.is_dangerous(me, &self.constants))
        {
            return self.value_projectiles(position) * 3.0
                + self.value_outside(position) * 5.0
                + self.value_shooting_sounds(position)
                + self.value_enemies(position, me, fight_mode);
        }

        self.value_zone(position)
            + self.value_outside(position) * 5.0
            + self.value_hit_sounds(position)
            + self.value_steps_sounds(position)
            + self.value_enemies(position, me, fight_mode)
            + self.value_allies(position, me)
            + self.value_loot(position, me)
    }

    pub fn value_unspawned(&self, position: Vec2, me: &Unit) -> f64 {
        self.value_zone(position) * 5.0
            + self.value_outside(position) * 5.0
            + self.value_shooting_sounds(position)
            + self.value_hit_sounds(position)
            + self.value_steps_sounds(position)
            + self.value_enemies(position, me, FightMode::RunWithNoWeapons)
            + self.value_allies(position, me)
            + self.value_loot(position, me)
    }
}
