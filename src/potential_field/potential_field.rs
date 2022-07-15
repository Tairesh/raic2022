use crate::model::*;
use std::collections::HashSet;
use std::f64::consts::PI;
use std::ops::Not;

pub struct PotentialField {
    constants: Constants,
    seeing_units: Vec<Unit>,
    seeing_projectiles: Vec<Projectile>,
    pub dangerous_projectiles: Vec<Projectile>,
    pub old_projectiles: Vec<Projectile>,
    zone: Zone,
    pub shooting_sounds: Vec<(Sound, Vec2, i32)>,
    pub hit_sounds: Vec<(Sound, i32)>,
    pub steps_sounds: Vec<(Sound, i32)>,
    current_tick: i32,
    my_id: i32,
}

impl PotentialField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            constants: constants.clone(),
            seeing_units: Vec::new(),
            seeing_projectiles: Vec::new(),
            old_projectiles: Vec::new(),
            dangerous_projectiles: Vec::new(),
            zone: Zone::default(),
            shooting_sounds: Vec::new(),
            hit_sounds: Vec::new(),
            steps_sounds: Vec::new(),
            current_tick: 0,
            my_id: 0,
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.current_tick = game.current_tick;
        self.my_id = game.my_id;
        self.seeing_units = game.units.clone();
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
            .filter(|projectile| projectile.is_dangerous(game, &self.constants))
            .cloned()
            .collect();
        self.dangerous_projectiles.extend(
            self.old_projectiles
                .iter()
                .filter(|projectile| projectile.is_dangerous(game, &self.constants))
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
                            let distance = unit.position.distance(&sound.position);
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
                            let distance = unit.position.distance(&sound.position);
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
                            let distance = unit.position.distance(&sound.position);
                            let props = &self.constants.sounds[sound.type_index as usize];
                            distance <= self.constants.unit_radius + props.offset
                        })
                })
                .map(|sound| (sound.clone(), game.current_tick)),
        );
        self.steps_sounds
            .retain(|&(.., tick)| game.current_tick - tick < 50);
    }

    pub fn points_around(&self, unit_id: i32) -> Vec<Vec2> {
        let me = self.seeing_units.iter().find(|u| u.id == unit_id).unwrap();
        // where unit will be in 3 ticks
        let inertion = (me.velocity / self.constants.ticks_per_second) * 3.0;
        let position = me.position + inertion;

        (0..8)
            .map(|i| {
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
                        .any(|u| u.position.distance(p) < self.constants.unit_radius * 2.0)
                        .not()
            })
            .collect()
    }

    fn value_projectiles(&self, position: &Vec2) -> f64 {
        let mut value = 0.0;
        for projectile in self.dangerous_projectiles.iter() {
            let line = projectile.as_line();

            let distance = line.distance_to_point(&position);
            let distance_limit = self.constants.unit_radius * 5.0;

            if distance > distance_limit {
                continue;
            }

            value -= 1.0 - distance / distance_limit;
            let distance = projectile.position.distance(position);
            let range = projectile.range();

            value += 0.05 * distance / range;
        }
        value
    }

    fn value_zone(&self, position: &Vec2) -> f64 {
        let distance = self.zone.next_center.distance(position);
        let wanna_radius = (self.zone.next_radius - self.constants.unit_radius * 4.0)
            .max(self.constants.unit_radius * 2.0);
        if distance < wanna_radius {
            return distance / wanna_radius;
        }

        1.0 - distance / wanna_radius
    }

    fn value_outside(&self, position: &Vec2) -> f64 {
        let distance = self.zone.current_center.distance(position);
        if self.zone.current_radius - distance < self.constants.unit_radius * 2.0 {
            return -distance / self.zone.current_radius;
        }

        0.0
    }

    fn value_shooting_sounds(&self, position: &Vec2) -> f64 {
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

    fn value_hit_sounds(&self, position: &Vec2) -> f64 {
        let mut value = 0.0;
        for (sound, tick) in self.hit_sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;
            let distance = sound.position.distance(position);
            let distance_limit = self.constants.unit_radius * 5.0;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * tick_k;
        }
        value
    }

    fn value_steps_sounds(&self, position: &Vec2) -> f64 {
        let mut value = 0.0;
        for (sound, tick) in self.steps_sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;
            let distance = sound.position.distance(position);
            let distance_limit = self.constants.unit_radius * 5.0;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * tick_k;
        }
        value
    }

    fn value_enemies(&self, position: &Vec2) -> f64 {
        let mut value = 0.0;

        // TODO: запоминать врагов
        for enemy in self
            .seeing_units
            .iter()
            .filter(|u| u.player_id != self.my_id)
        {
            if enemy.weapon.is_none() {
                continue;
            }

            let weapon = enemy.weapon.unwrap() as usize;

            if enemy.ammo[weapon] == 0 {
                continue;
            }

            let prop = &self.constants.weapons[weapon];
            let range = prop.projectile_life_time * prop.projectile_speed;

            let aim = Line::new(
                enemy.position,
                enemy.position + enemy.direction.normalize() * range,
            );

            let distance = aim.distance_to_point(&position);
            let distance_limit = range;
            if distance > distance_limit {
                continue;
            }

            value -= (1.0 - distance / distance_limit) * 0.5;
        }

        value
    }

    pub fn is_in_danger(&self, me: &Unit) -> bool {
        !self.dangerous_projectiles.is_empty()
            || !self.shooting_sounds.is_empty()
            || !self.hit_sounds.is_empty()
            || !self.steps_sounds.is_empty()
            || me.position.distance(&self.zone.current_center)
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

    pub fn value(&self, position: &Vec2) -> f64 {
        if !self.dangerous_projectiles.is_empty() {
            return self.value_projectiles(position) + self.value_outside(position);
        }

        self.value_zone(position)
            + self.value_outside(position)
            + self.value_shooting_sounds(position)
            + self.value_hit_sounds(position)
            + self.value_steps_sounds(position)
            + self.value_enemies(position)
    }
}