use crate::debug_interface::DebugInterface;
// use ai_cup_22::debugging::Color;
use ai_cup_22::model::*;

pub struct MyStrategy {
    constants: Constants,
    last_look_around: i32,
    looking_around: i32,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        Self {
            constants,
            last_look_around: 0,
            looking_around: 0,
        }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        let enemies: Vec<&Unit> = game
            .units
            .iter()
            .filter(|u| u.player_id != game.my_id)
            .collect();

        // if let Some(debug_interface) = debug_interface {
        //     let me = game
        //         .units
        //         .iter()
        //         .filter(|u| u.player_id == game.my_id)
        //         .next()
        //         .unwrap();
        //     let position = me.position;
        //     let line = Line::new(position, position + (me.direction * 100.0));
        //     debug_interface.add_poly_line(
        //         vec![line.start, line.end],
        //         1.0,
        //         Color::new(0.0, 0.0, 1.0, 1.0),
        //     );
        //     for o in self.constants.obstacles.iter().cloned() {
        //         let circle = Circle::new(o.position, o.radius);
        //         debug_interface.add_circle(
        //             o.position,
        //             o.radius,
        //             if circle.intercept_with_line(&line) {
        //                 Color::new(1.0, 0.0, 0.0, 1.0)
        //             } else {
        //                 Color::new(0.0, 1.0, 0.0, 1.0)
        //             },
        //         );
        //     }
        // }

        Order {
            unit_orders: game
                .units
                .iter()
                .filter(|u| u.player_id == game.my_id)
                .map(|u| {
                    let closest_enemy = enemies
                        .iter()
                        .filter(|e| {
                            !self
                                .constants
                                .obstacles
                                .iter()
                                .filter(|o| !o.can_shoot_through)
                                .any(|o| {
                                    let line = Line::new(u.position, e.position);
                                    let circle = Circle::new(
                                        o.position,
                                        o.radius - self.constants.unit_radius,
                                    );
                                    circle.intercept_with_line(&line)
                                })
                        })
                        .min_by(|a, b| {
                            let a_dist = u.position.square_distance(&a.position);
                            let b_dist = u.position.square_distance(&b.position);
                            a_dist.partial_cmp(&b_dist).unwrap()
                        });

                    let target_direction = if let Some(closest_enemy) = closest_enemy {
                        closest_enemy.position + closest_enemy.velocity * 0.3 - u.position
                    } else if game.current_tick - self.last_look_around > 500
                        || self.looking_around < 100
                    {
                        self.last_look_around = game.current_tick;
                        self.looking_around += 1;
                        Vec2::new(-u.direction.y, u.direction.x)
                    } else {
                        self.looking_around = 0;
                        game.zone.next_center - u.position
                    }
                    .normalize();

                    (
                        u.id,
                        UnitOrder {
                            target_velocity: (game.zone.next_center - u.position).normalize()
                                * self.constants.max_unit_forward_speed,
                            target_direction,
                            action: if let Some(closest_enemy) = closest_enemy {
                                let d = closest_enemy.position.distance(&u.position);

                                let my_weapon_range = self.constants.weapons
                                    [u.weapon.unwrap() as usize]
                                    .projectile_speed
                                    * self.constants.weapons[u.weapon.unwrap() as usize]
                                        .projectile_life_time
                                    + self.constants.unit_radius;

                                Some(ActionOrder::Aim {
                                    shoot: d <= my_weapon_range,
                                })
                            } else if let Some(loot) = game
                                .loot
                                .iter()
                                .filter(|l| {
                                    l.position.distance(&u.position) < self.constants.unit_radius
                                })
                                .next()
                            {
                                Some(ActionOrder::Pickup { loot: loot.id })
                            } else if u.shield < self.constants.max_shield && u.shield_potions > 0 {
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
