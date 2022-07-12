use crate::debugging::Color;
use crate::model::{
    Circle, Constants, Game, Item, Line, Loot, Projectile, Unit, Vec2, Vec2i, Zone,
};

pub trait PotentialField {
    fn value(&self, point: Vec2) -> f64;
}

pub struct ZoneField {
    zone: Zone,
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
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.zone = game.zone.clone();
    }
}

impl PotentialField for ZoneField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        let current_distance = point.distance(&self.zone.current_center);
        if current_distance >= self.zone.current_radius - 2.0 {
            value += 1.0 - current_distance / self.zone.current_radius;
        }
        let distance = point.distance(&self.zone.next_center);
        if distance < self.zone.next_radius {
            value -= distance / self.zone.next_radius;
        } else {
            value -= (distance - self.zone.next_radius) / (self.zone.next_radius * 2.0);
        }

        value
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

        1.0
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
            let aim = Line::new(e.position, e.position + e.direction.normalize() * 100.0);
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
            value += d / 10_000.0;
        }
        value.clamp(-1.0, 1.0)
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
        self.loot = game
            .loot
            .iter()
            .filter(|l| !matches!(l.item, Item::Weapon { type_index: 0 }))
            .cloned()
            .collect();
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
        let mut value = -1.0;
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
                value += 2.0
            } else if loot.position.square_distance(&point) < self.unit_radius * 4.0 {
                value += 1.5
            } else if loot.position.square_distance(&point) < self.unit_radius * 6.0 {
                value += 1.0
            } else if loot.position.square_distance(&point) < self.unit_radius * 8.0 {
                value += 0.5
            }
        }

        value
    }
}

pub struct SoundsField {
    sounds: Vec<Vec2>,
}

impl SoundsField {
    pub fn new() -> Self {
        Self { sounds: vec![] }
    }

    pub fn update(&mut self, game: &Game) {
        self.sounds = game.sounds.iter().map(|s| s.position).collect();
    }
}

impl PotentialField for SoundsField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = 0.0;
        for b in self.sounds.iter() {
            if b.square_distance(&point) < 500.0 {
                value -= 1.0
            } else if b.square_distance(&point) < 2500.0 {
                value -= 0.5
            } else if b.square_distance(&point) < 5000.0 {
                value -= 0.25
            }
        }

        value
    }
}

pub struct ProjectilesField {
    projectiles: Vec<Projectile>,
    unit_radius: f64,
}

impl ProjectilesField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            projectiles: vec![],
            unit_radius: constants.unit_radius,
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.projectiles = game
            .projectiles
            .iter()
            .filter(|p| p.shooter_player_id != game.my_id)
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
            let d = line.distance_to_point(&point);
            if d < 1.0 {
                return -1.0;
            } else if d < 2.0 {
                value -= 0.75;
            } else if d < 3.0 {
                value -= 0.5;
            } else if d < 4.0 {
                value -= 0.25;
            } else if d < 5.0 {
                value -= 0.1;
            }
        }

        value
    }
}

pub struct PotentialFields {
    pub zone: ZoneField,
    pub obstacles: ObstacleField,
    pub enemies: EnemyField,
    pub loot: LootField,
    pub sounds: SoundsField,
    pub projectiles: ProjectilesField,
}

impl PotentialFields {
    pub fn new(constants: &Constants) -> Self {
        Self {
            zone: ZoneField::new(constants),
            obstacles: ObstacleField::new(constants),
            enemies: EnemyField::empty(),
            loot: LootField::new(constants),
            sounds: SoundsField::new(),
            projectiles: ProjectilesField::new(constants),
        }
    }

    pub fn calculate(&mut self, game: &Game, _constants: &Constants) {
        self.zone.update(game);
        self.enemies = EnemyField::new(game);
        self.loot.update(game);
        self.sounds.update(game);
        self.projectiles.update(game);
    }

    pub fn value(&self, point: Vec2i, battle_mode: bool, outside: bool) -> f64 {
        let point = point.into();
        let mut value = self.zone.value(point);
        value += self.obstacles.value(point);
        value += self.projectiles.value(point) * 5.0;
        value += self.enemies.value(point);
        if outside {
            return value;
        }
        value += self.sounds.value(point);
        if battle_mode {
            value += self.loot.value(point);
        }
        value / if battle_mode { 10.0 } else { 9.0 }
    }
}

pub fn color_by_value(value: f64) -> Color {
    let value = (value + 1.0) / 2.0;
    let r = f64::min(1.0, 1.0 - value);
    let g = f64::min(1.0, value);
    let b = 0.0;

    Color::new(r, g, b, 0.5)
}
