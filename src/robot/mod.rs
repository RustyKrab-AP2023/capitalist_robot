pub(crate) mod robot_impl;
mod modes;

use std::collections::{HashMap, VecDeque};
use std::fs::{File};
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
use charting_tools::charted_map::ChartedMap;
use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{debug, Direction, look_at_sky, robot_map};
use robotics_lib::runner::{Robot, Runnable, backpack::BackPack};
use robotics_lib::world::{coordinates::Coordinate, tile::Content, World};
use robotics_lib::world::environmental_conditions::EnvironmentalConditions;
use robotics_lib::world::tile::Tile;

use shared_state::{SharedState, StateUpdate, GameWorldUpdate};

pub(crate) enum Mode{
    SearchingContent,
    ScanContent,
    SearchingBank,
    FollowStreet,
    ScanBank
}
pub struct MyRobot{
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
    pub(crate) file: File,
    pub(crate) local_shared_state: SharedState,

    pub shared_state_rc: Rc<RefCell<SharedState>>,
}

impl Runnable for MyRobot{
    fn process_tick(&mut self, world: &mut World) {
        if self.terminated{
           return;
        }

        self.debug_file();

        self.tick_init(world);

        match self.mode {
            Mode::ScanContent => {modes::scan_content::run_scan_content_mode(self, world);}
            Mode::SearchingContent => {modes::searching_content::run_searching_content_mode(self, world);}
            Mode::SearchingBank => {modes::searching_bank::run_searching_bank_mode(self, world);}
            Mode::FollowStreet => {modes::follow_street::run_follow_street_mode(self, world);}
            Mode::ScanBank => {modes::scan_bank::run_scan_bank_mode(self, world);}
        }
        
        self.tick_finish(world);
    }

    fn handle_event(&mut self, event: Event) {

        match event {
            Event::Ready => {}
            Event::Terminated => {}
            Event::TimeChanged(_) => {}
            Event::DayChanged(_) => {
                self.shared_event_update(event);
            }
            Event::EnergyRecharged(_) => {
                self.shared_event_update(event);

            }
            Event::EnergyConsumed(_) => {
                self.shared_event_update(event);
            }
            Event::Moved(_, _) => {
                self.shared_event_update(event);
            }
            Event::TileContentUpdated(_, _) => {
                self.shared_event_update(event);

            }
            Event::AddedToBackpack(_, _) => {
                self.shared_event_update(event);
            }
            Event::RemovedFromBackpack(_, _) => {
                self.shared_event_update(event);
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
