use std::fs::OpenOptions;
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;

use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_map::ChartedMap;
use charting_tools::ChartingTools;
use robotics_lib::event::events::Event;
use robotics_lib::interface::{debug, destroy, go, robot_map, robot_view, Direction};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;

use crate::robot::{Mode, MyRobot};
use crate::utils::{enough_distant, get_coords_row_col, go_where_you_can, look_around};

use shared_state::{SharedState, StateUpdate, GameWorldUpdate};

impl MyRobot{
    pub fn new(shared_state_rc: Rc<RefCell<SharedState>>)-> Self{
        Self{
            robot: Robot::new(),
            last_world: None,
            chart: ChartingTools::tool::<ChartedMap<Content>>().unwrap(),
            mode: Mode::SearchingContent,
            direction: Direction::Right,
            count_tick: 0,
            rounds: 0,
            avoid_street: false,
            no_more_discovery: false,
            expl_pause_tick_time: 0,
            explorer_pause: false,
            terminated: false,
            file: OpenOptions::new().create(true).write(true).append(true).open(&"logs.txt").unwrap(),
            local_shared_state: SharedState::new(),
            shared_state_rc,
        }
    }

    ///initializes the tick
    pub(crate) fn tick_init(&mut self, world: &mut World){
        
        // Cleaning updates queue from local shared state if used before
        self.local_shared_state.clear();

        // Updating local shared state
        let (world_map, _world_dimension, robot_pos) = debug(self, world);
        let robot_map = robot_map(&world).unwrap();

        let mut update = GameWorldUpdate::default();
        update.set_world(&world_map);
        update.set_robot_pos(robot_pos);
        update.set_robot_map(&robot_map);
        let update = StateUpdate::GameWorld(update);

        self.local_shared_state.add_update(update);

        //switch between SearchingContent and ScanContent
        if let Mode::SearchingContent=self.mode{
            if !self.explorer_pause{
                self.count_tick= (self.count_tick+1)%usize::MAX;
                if (self.count_tick%50)==0{
                    self.mode = Mode::ScanContent;
                }
            }
        }

        //explorer_pause check
        if self.explorer_pause{
            //self.expl_pause_tick_time = self.expl_pause_tick_time +1;
            if self.expl_pause_tick_time > 50 {
                self.mode=Mode::SearchingBank;
                self.explorer_pause=false;
                self.expl_pause_tick_time =0;
            }
        }



        //when the energy reach a threshold
        if self.robot.energy.get_energy_level() < 300 {
            *self.get_energy_mut()=Dynamo::update_energy();
        }

        //checks if the robot is enough distant from discovered banks or probably close streets and if the robot can scan again
        if enough_distant(self.get_coordinate(), &self.chart) && !self.no_more_discovery{
            self.avoid_street=false;
        }else{
            self.avoid_street=true;
            self.rounds=0;
            if let Mode::FollowStreet=self.mode{
                self.sblock(world);
                self.mode=Mode::SearchingContent;
            }
        }
    }

    pub(crate) fn tick_finish(&mut self, world: &mut World) {
        // Updating local shared state
        let (world_map, _world_dimension, robot_pos) = debug(self, world);
        let robot_map = robot_map(&world).unwrap();

        let mut update = GameWorldUpdate::default();
        update.set_world(&world_map);
        update.set_robot_map(&robot_map);
        let update = StateUpdate::GameWorld(update);

        self.local_shared_state.add_update(update);
        
        // Updating shared state by replacing it with local one
        self.shared_state_rc.replace(self.local_shared_state.clone());
    }
    
    pub(crate) fn shared_event_update(&mut self, event: Event) {
        // Updating local shared state
        let update = StateUpdate::GameEvent(event);
        self.local_shared_state.add_update(update);
    }

    ///save a tile given the direction
    pub(crate) fn save_tile(&mut self, direction: &Direction, content:&Content){
        let direction_coord=get_coords_row_col(self, direction);
        self.chart.save(content, &ChartedCoordinate(direction_coord.0, direction_coord.1));
    }

    ///destroy the content around the robot and if it finds a street enters in FollowStreet mode
    pub(crate) fn check_content(&mut self, world: &mut World, direction: &Direction, t:&Tile){

        //rock, tree, garbage, coin, fish
        if [0, 2, 4, 10].contains(&t.content.index()){ //tolto 1:Tree, 4: Coin
            let _ =destroy(self, world, direction.clone());
        }

        //if is not a city              bank, market, building
        if t.tile_type==TileType::Street && !self.avoid_street{
            self.direction=direction.clone();
            self.mode=Mode::FollowStreet;
        }

    }

    ///move the robot until it is in a non Street tile
    pub(crate) fn sblock(&mut self, world: &mut World){
        let mut direction=look_around(self, world);

        while let None=direction{
            go_where_you_can(self, world);
            direction= look_around(self, world);
        }

        self.direction=direction.unwrap().clone();
        //if it has not enough energy
        if let Err(_)=go(self, world, self.direction.clone()){
            *self.get_energy_mut()=Dynamo::update_energy();
        }
    }


    pub(crate) fn debug_file(&mut self){
        let _ = self.file.write_all(self.robot.energy.get_energy_level().to_string().as_bytes());
        let _ = match self.mode {
            Mode::ScanBank => {self.file.write_all("\t|\tMod: ScanningBank\t\t|\t".as_bytes())}
            Mode::ScanContent => {self.file.write_all("\t|\tMod: ScanningContent\t\t|\t".as_bytes())}
            Mode::SearchingContent => {self.file.write_all("\t|\tMod: SearchingContent\t\t|\t".as_bytes())}
            Mode::SearchingBank => {self.file.write_all("\t|\tMod: SearchingBank\t\t|\t".as_bytes())}
            Mode::FollowStreet => {self.file.write_all("\t|\tMod: FollowStreet\t\t|\t".as_bytes())}
        };
        let mut string;
        for (content, size) in self.robot.backpack.get_contents().iter(){
            match content {
                Content::Coin(0) => {
                    string="Coin: ".to_string();
                    string.push_str(&size.to_string());
                    string.push_str(&"  ");
                    let _ = self.file.write_all(string.as_bytes());
                }
                Content::Rock(0) => {
                    string="Rock: ".to_string();
                    string.push_str(&size.to_string());
                    string.push_str(&"  ");
                    let _ = self.file.write_all(string.as_bytes());
                }
                Content::Tree(0) => {
                    string="Tree: ".to_string();
                    string.push_str(&size.to_string());
                    string.push_str(&"  ");
                    let _ = self.file.write_all(string.as_bytes());
                }
                Content::Fish(0) => {
                    string="Fish: ".to_string();
                    string.push_str(&size.to_string());
                    string.push_str(&"  ");
                    let _ = self.file.write_all(string.as_bytes());
                }
                Content::Garbage(0) => {
                    string="Garbage: ".to_string();
                    string.push_str(&size.to_string());
                    string.push_str(&"  ");
                    let _ = self.file.write_all(string.as_bytes());
                }
                _ => {}
            }
        }
        let _ = self.file.write_all("\n\n".as_bytes());
    }
}
