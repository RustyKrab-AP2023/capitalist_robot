pub(crate) mod robot_impl;
mod modes;

use std::fs::{File};
use std::io::Write;
use charting_tools::charted_map::ChartedMap;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{Direction};
use robotics_lib::runner::{Robot, Runnable, backpack::BackPack};
use robotics_lib::world::{coordinates::Coordinate, tile::Content, World};
use robotics_lib::world::tile::Tile;

use shared_state::{SharedStateWrapper};

pub(crate) enum Mode{
    SearchingContent,
    ScanContent,
    SearchingBank,
    FollowStreet,
    ScanBank
}
pub struct CapitalistRobot {
    pub(crate) robot: Robot,
    pub(crate) last_world: Option<Vec<Vec<Tile>>>,
    pub(crate) chart: ChartedMap<Content>,
    pub(crate) mode: Mode,
    pub(crate) direction: Direction,
    pub(crate) count_tick: usize,
    pub(crate) rounds: usize,
    pub(crate) avoid_street: bool,
    pub(crate) no_more_discovery: bool,
    pub(crate) expl_pause_tick_time: usize,
    pub(crate) explorer_pause: bool,
    pub(crate) terminated: bool,
    pub(crate) first_tick: bool,
    pub(crate) shared_state: SharedStateWrapper
}

impl Runnable for CapitalistRobot {
    fn process_tick(&mut self, world: &mut World) {
        if self.terminated{
           return;
        }

        self.tick_init(world);

        match self.mode {
            Mode::ScanContent => {modes::scan_content::run_scan_content_mode(self, world);}
            Mode::SearchingContent => {modes::searching_content::run_searching_content_mode(self, world);}
            Mode::SearchingBank => {modes::searching_bank::run_searching_bank_mode(self, world);}
            Mode::FollowStreet => {modes::follow_street::run_follow_street_mode(self, world);}
            Mode::ScanBank => {modes::scan_bank::run_scan_bank_mode(self, world);}
        }
    }

    fn handle_event(&mut self, event: Event) {

        match event {
            Event::Ready => {}
            Event::Terminated => {}
            Event::TimeChanged(_) => {
                self.shared_state.update_event(event);
            }
            Event::DayChanged(_) => {
                self.shared_state.update_event(event);
            }
            Event::EnergyRecharged(_) => {
                self.shared_state.update_event(event);
            }
            Event::EnergyConsumed(_) => {
                self.shared_state.update_event(event);
            }
            Event::Moved(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::TileContentUpdated(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::AddedToBackpack(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::RemovedFromBackpack(_, _) => {
                self.shared_state.update_event(event);
            }
        }
    }

    fn get_energy(&self) -> &Energy {
        &self.robot.energy
    }

    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }

    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }

    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }

    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}
