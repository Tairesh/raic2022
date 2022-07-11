use crate::debugging::Color;
use crate::model::{Circle, Constants, Game, Item, Loot, Unit, Vec2};

pub trait PotentialField {
    fn value(&self, point: Vec2) -> f64;
}

pub struct ZoneField {
    zone_center: Vec2,
    square_zone_radius: f64,
}

impl PotentialField for ZoneField {
    fn value(&self, point: Vec2) -> f64 {
        let distance = point.square_distance(&self.zone_center);
        if distance < self.square_zone_radius {
            1.0
        } else {
            -(distance - self.square_zone_radius) / self.square_zone_radius
        }
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
            if e.weapon.is_none() || e.ammo[e.weapon.unwrap() as usize] == 0 {
                value += 0.1;
            } else if e.position.square_distance(&point) < 300.0 {
                value += -1.0;
            } else if e.position.square_distance(&point) < 700.0 {
                value += -0.75;
            } else if e.position.square_distance(&point) < 1000.0 {
                value += -0.5;
            } else if e.position.square_distance(&point) < 1500.0 {
                value += -0.25;
            }
        }
        value.clamp(-1.0, 1.0)
    }
}

pub struct LootField {
    loot: Vec<Loot>,
    unit_radius: f64,
}

impl LootField {
    pub fn new(constants: &Constants) -> Self {
        Self {
            loot: vec![],
            unit_radius: constants.unit_radius,
        }
    }

    pub fn update(&mut self, game: &Game) {
        self.loot = game
            .loot
            .iter()
            .filter(|l| !matches!(l.item, Item::Weapon { type_index: 0 }))
            .cloned()
            .collect();
    }
}

impl PotentialField for LootField {
    fn value(&self, point: Vec2) -> f64 {
        let mut value = -1.0;
        for loot in self.loot.iter() {
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
            if b.square_distance(&point) < 100.0 {
                value -= 1.0
            } else if b.square_distance(&point) < 500.0 {
                value -= 0.5
            } else if b.square_distance(&point) < 1000.0 {
                value -= 0.25
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
}

impl PotentialFields {
    pub fn new(constants: &Constants) -> Self {
        Self {
            zone: ZoneField {
                zone_center: Vec2::new(0.0, 0.0),
                square_zone_radius: 300.0 * 300.0,
            },
            obstacles: ObstacleField::new(constants),
            enemies: EnemyField::empty(),
            loot: LootField::new(constants),
            sounds: SoundsField::new(),
        }
    }

    pub fn calculate(&mut self, game: &Game, _constants: &Constants) {
        if game.zone.next_center != self.zone.zone_center {
            self.zone = ZoneField {
                zone_center: game.zone.next_center,
                square_zone_radius: game.zone.next_radius * game.zone.next_radius,
            };
        }
        self.enemies = EnemyField::new(game);
        self.loot.update(game);
        self.sounds.update(game);
    }

    pub fn value(&self, point: Vec2, include_loot: bool) -> f64 {
        let mut value = self.zone.value(point);
        value += self.obstacles.value(point);
        value += self.enemies.value(point);
        if include_loot {
            value += self.loot.value(point);
        }
        value += self.sounds.value(point);
        value / if include_loot { 5.0 } else { 4.0 }
    }
}

pub fn color_by_value(value: f64) -> Color {
    let value = (value + 1.0) / 2.0;
    let r = f64::min(1.0, 1.0 - value);
    let g = f64::min(1.0, value);
    let b = 0.0;

    Color::new(r, g, b, 0.5)
}
