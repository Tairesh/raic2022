use crate::debugging::Color;
use crate::model::{
    Circle, Constants, Game, Item, Line, Loot, Projectile, SoundProperties, Unit, Vec2, Vec2i, Zone,
};
use std::collections::VecDeque;

pub trait PotentialField {
    fn value(&self, point: Vec2) -> f64;
}

pub struct ZoneField {
    zone: Zone,
    // unit_radius: f64,
    // my_position: Vec2,
}

impl ZoneField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            zone: Zone {
                current_center: Vec2::zero(),
                current_radius: constants.initial_zone_radius,
                next_center: Vec2::zero(),
                next_radius: constants.initial_zone_radius,
            },
            // unit_radius: constants.unit_radius,
            // my_position: Vec2::zero(),
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.zone = game.zone.clone();
        // let me = game.units.iter().find(|u| u.id == game.my_id).unwrap();
        // self.my_position = me.position.clone();
    }
}

impl PotentialField for ZoneField {
    fn value(&self, point: Vec2) -> f64 {
        let distance = point.distance(&self.zone.current_center);
        let k = if distance > self.zone.next_radius {
            10.0
        } else {
            1.0
        };
        k * -distance / self.zone.current_radius
    }
}

pub struct ObstacleField {
    obstacles: Vec<Circle>,
}

impl ObstacleField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            obstacles: constants
                .obstacles
                .iter()
                .map(|o| Circle::new(o.position, o.radius + constants.unit_radius))
                .collect(),
        }
    }
}

impl PotentialField for ObstacleField {
    fn value(&self, point: Vec2) -> f64 {
        for o in self.obstacles.iter() {
            if o.contains_point(&point) {
                return -1.0;
            }
        }

        0.0
    }
}

pub struct EnemyField {
    enemies: Vec<Unit>,
}

impl EnemyField {
    pub fn empty() -> Self {
        Self { enemies: vec![] }
    }

    pub fn new(game: &Game) -> Self {
        Self {
            enemies: game
                .units
                .iter()
                .filter(|u| u.player_id != game.my_id)
                .cloned()
                .collect(),
        }
    }
}

impl PotentialField for EnemyField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value: f64 = 0.0;
        for e in self.enemies.iter() {
            let aim = Line::new(e.position, e.position + e.direction.normalize() * 50.0);
            let d = aim.distance_to_point(&point);
            if e.weapon.is_none() || e.ammo[e.weapon.unwrap() as usize] == 0 {
                value += 1.0;
            } else if d < 1.0 {
                value += -1.0;
            } else if d < 2.0 {
                value += -0.75;
            } else if d < 3.0 {
                value += -0.5;
            } else if d < 4.0 {
                value += -0.25;
            }

            let d = e.position.square_distance(&point);
            if d > 2_500.0 {
                continue;
            }
            value += d / 2_500.0;
        }
        value.clamp(-1.0, 0.0)
    }
}

pub struct LootField {
    loot: Vec<Loot>,
    constants: Constants,
    me: Unit,
    unit_radius: f64,
}

impl LootField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            loot: vec![],
            unit_radius: constants.unit_radius,
            me: Unit {
                position: Vec2::zero(),
                remaining_spawn_time: None,
                velocity: Vec2::zero(),
                direction: Vec2::zero(),
                aim: 0.0,
                action: None,
                health_regeneration_start_tick: 0,
                weapon: None,
                next_shot_tick: 0,
                ammo: vec![0; constants.weapons.len()],
                player_id: 0,
                health: 0.0,
                shield: 0.0,
                id: 0,
                extra_lives: 0,
                shield_potions: 0,
            },
            constants: constants.clone(),
        }
    }

    pub fn update(&mut self, game: &Game) {
        // TODO: remember loot
        self.loot = game
            .loot
            .iter()
            .filter(|l| !matches!(l.item, Item::Weapon { type_index: 0 }))
            .cloned()
            .collect();
        // TODO: multiunit support
        self.me = game
            .units
            .iter()
            .find(|u| u.player_id == game.my_id)
            .unwrap()
            .clone();
    }
}

impl PotentialField for LootField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        for loot in self.loot.iter().filter(|l| {
            let t = &l.item;
            match t {
                Item::Weapon { type_index } => {
                    if *type_index == 0 {
                        return false;
                    }
                    if let Some(weapon_id) = self.me.weapon {
                        if *type_index == weapon_id || self.me.ammo[*type_index as usize] < 10 {
                            return false;
                        }
                        let my_weapon = &self.constants.weapons[weapon_id as usize];
                        let new_weapon = &self.constants.weapons[*type_index as usize];
                        if my_weapon.projectile_life_time * my_weapon.projectile_speed
                            > new_weapon.projectile_life_time * new_weapon.projectile_speed
                        {
                            return false;
                        }
                    }
                }
                Item::Ammo {
                    weapon_type_index, ..
                } => {
                    if self.me.ammo[*weapon_type_index as usize]
                        == self.constants.weapons[*weapon_type_index as usize].max_inventory_ammo
                    {
                        return false;
                    }
                }
                Item::ShieldPotions { .. } => {
                    if self.me.shield_potions == self.constants.max_shield_potions_in_inventory {
                        return false;
                    }
                }
            }
            true
        }) {
            // TODO: check loot type

            if loot.position.square_distance(&point) < self.unit_radius * 2.0 {
                value += 1.0
            } else if loot.position.square_distance(&point) < self.unit_radius * 4.0 {
                value += 0.75
            } else if loot.position.square_distance(&point) < self.unit_radius * 6.0 {
                value += 0.5
            } else if loot.position.square_distance(&point) < self.unit_radius * 8.0 {
                value += 0.25
            }
        }

        value
    }
}

pub struct ShootingSoundsField {
    sounds: Vec<(i32, Vec2, Vec2)>,
    current_tick: i32,
    sound_properties: Vec<SoundProperties>,
    my_coordinates: Vec2,
    unit_radius: f64,
}

impl ShootingSoundsField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            sounds: vec![],
            current_tick: 0,
            sound_properties: constants.sounds.clone(),
            my_coordinates: Vec2::zero(),
            unit_radius: constants.unit_radius,
        }
    }

    pub fn update(&mut self, game: &Game) {
        let me = game
            .units
            .iter()
            .find(|u| u.player_id == game.my_id)
            .unwrap();
        self.my_coordinates = me.position;
        self.sounds.extend(
            game.sounds
                .iter()
                .filter(|s| {
                    if game
                        .units
                        .iter()
                        .filter(|u| u.player_id != game.my_id)
                        .any(|u| {
                            let distance = u.position.distance(&s.position);
                            distance < self.unit_radius * 2.0
                        })
                    {
                        return false;
                    }

                    let name = &self.sound_properties[s.type_index as usize].name;
                    match name.as_str() {
                        "Wand" | "Staff" | "Bow" => true,
                        _ => false,
                    }
                })
                .map(|s| (game.current_tick, s.position, me.position.clone())),
        );
        self.sounds
            .retain(|&(tick, ..)| game.current_tick - tick < 50);
        self.current_tick = game.current_tick;
    }
}

impl PotentialField for ShootingSoundsField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        for (tick, place, my_pos) in self.sounds.iter() {
            let tick_k = 1.0 - (self.current_tick - tick) as f64 / 50.0;

            let line = Line::new(*place, *my_pos);
            let distance = line.distance_to_point(&point);
            if distance > self.unit_radius * 2.0 {
                continue;
            }

            value -= (1.0 - distance / (self.unit_radius * 2.0)) * tick_k;
        }
        value
    }
}

pub struct ProjectilesField {
    projectiles: Vec<Projectile>,
    unit_radius: f64,
    // damage: Vec<f64>,
    // obstacles: Vec<Obstacle>,
}

impl ProjectilesField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            projectiles: vec![],
            unit_radius: constants.unit_radius,
            // obstacles: constants.obstacles.clone(),
            // damage: constants
            //     .weapons
            //     .iter()
            //     .map(|w| w.projectile_damage)
            //     .collect(),
        }
    }

    pub fn update(&mut self, game: &Game) {
        let me = game
            .units
            .iter()
            .find(|u| u.player_id == game.my_id)
            .unwrap();
        self.projectiles = game
            .projectiles
            .iter()
            .filter(|projectile| {
                let moving = projectile.velocity * projectile.life_time + self.unit_radius;
                let line = Line::new(projectile.position, projectile.position + moving);
                let distance = line.distance_to_point(&me.position);
                projectile.shooter_player_id != game.my_id && distance < self.unit_radius * 2.0
            })
            .cloned()
            .collect();
    }
}

impl PotentialField for ProjectilesField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        for projectile in self.projectiles.iter() {
            let moving = projectile.velocity * projectile.life_time + self.unit_radius;
            let line = Line::new(projectile.position, projectile.position + moving);

            let distance = line.distance_to_point(&point);
            if distance > 5.0 {
                continue;
            }

            value -= 1.0 - distance / 5.0;
        }

        value
    }
}

pub struct DontStayField {
    last_positions: VecDeque<Vec2>,
}

impl DontStayField {
    pub fn new() -> Self {
        Self {
            last_positions: VecDeque::with_capacity(10),
        }
    }

    pub fn update(&mut self, game: &Game) {
        let me = game
            .units
            .iter()
            .find(|u| u.player_id == game.my_id)
            .unwrap();
        if self.last_positions.len() == 10 {
            self.last_positions.pop_front();
        }
        self.last_positions.push_back(me.position);
    }
}

impl PotentialField for DontStayField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        for position in self.last_positions.iter() {
            let distance = position.distance(&point);
            if distance > 10.0 {
                continue;
            }
            value -= (1.0 - distance / 10.0) / 10.0;
        }
        value
    }
}

pub struct PotentialFields {
    pub zone: ZoneField,
    pub obstacles: ObstacleField,
    pub enemies: EnemyField,
    pub loot: LootField,
    pub sounds: ShootingSoundsField,
    pub projectiles: ProjectilesField,
    pub dont_stay: DontStayField,
}

impl PotentialFields {
    pub fn new(constants: &Constants) -> Self {
        Self {
            zone: ZoneField::new(constants),
            obstacles: ObstacleField::new(constants),
            enemies: EnemyField::empty(),
            loot: LootField::new(constants),
            sounds: ShootingSoundsField::new(constants),
            projectiles: ProjectilesField::new(constants),
            dont_stay: DontStayField::new(),
        }
    }

    pub fn calculate(&mut self, game: &Game, _constants: &Constants) {
        self.zone.update(game);
        self.enemies = EnemyField::new(game);
        self.loot.update(game);
        self.sounds.update(game);
        self.projectiles.update(game);
        self.dont_stay.update(game);
    }

    pub fn value(&self, point: Vec2i, battle_mode: bool) -> f64 {
        let point = point.into();
        let mut value = self.zone.value(point);
        value += self.obstacles.value(point);
        value += self.projectiles.value(point) * 5.0;
        value += self.enemies.value(point);
        value += self.sounds.value(point);
        if !battle_mode {
            value += self.loot.value(point);
            value += self.dont_stay.value(point);
        }
        value / if battle_mode { 11.0 } else { 9.0 }
    }
}

pub fn color_by_value(value: f64) -> Color {
    let value = (value + 1.0) / 2.0;
    let r = f64::min(1.0, 1.0 - value);
    let g = f64::min(1.0, value);
    let b = 0.0;

    Color::new(r, g, b, 0.5)
}
