use crate::debug_interface::DebugInterface;
use ai_cup_22::model::*;

pub struct MyStrategy {
    constants: Constants,
}

impl MyStrategy {
    pub fn new(constants: Constants) -> Self {
        // dbg!(&constants);
        Self { constants }
    }
    pub fn get_order(
        &mut self,
        game: &Game,
        _debug_interface: Option<&mut DebugInterface>,
    ) -> Order {
        Order {
            unit_orders: game
                .units
                .iter()
                .filter(|u| u.player_id == game.my_id)
                .map(|u| {
                    (
                        u.id,
                        UnitOrder {
                            target_velocity: (game.zone.next_center - u.position).normalize()
                                * self.constants.max_unit_forward_speed,
                            target_direction: (game.zone.next_center - u.position).normalize(),
                            action: None,
                        },
                    )
                })
                .collect(),
        }
    }
    pub fn debug_update(&mut self, _debug_interface: &mut DebugInterface) {}
    pub fn finish(&mut self) {}
}
