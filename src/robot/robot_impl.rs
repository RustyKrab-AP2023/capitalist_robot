use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_map::ChartedMap;
use charting_tools::ChartingTools;
use robotics_lib::interface::{debug, destroy, go, Direction};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;

use crate::robot::{Mode, CapitalistRobot};
use crate::utils::{enough_distant, get_coords_row_col, go_where_you_can, look_around};

use shared_state::{SharedStateWrapper};

impl CapitalistRobot {
    pub fn new(shared_state: SharedStateWrapper)-> Self{
        Self{
            robot: Robot::new(),
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
            first_tick: true,
            shared_state
        }
    }

    ///initializes the tick
    pub(crate) fn tick_init(&mut self, world: &mut World){

        //visualizer
        if self.first_tick{
            let debug_info = debug(self, world);
            self.shared_state.update_world(debug_info.0, debug_info.1, debug_info.2);
            self.first_tick = false;
        }

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

    ///save a tile given the direction
    pub(crate) fn save_tile(&mut self, direction: &Direction, content:&Content){
        let direction_coord=get_coords_row_col(self, direction);
        self.chart.save(content, &ChartedCoordinate(direction_coord.0, direction_coord.1));
    }

    ///destroy the content around the robot and if it finds a street enters in FollowStreet mode
    pub(crate) fn check_content(&mut self, world: &mut World, direction: &Direction, t:&Tile){

        //rock, garbage, fish
        if [0, 2, 4, 10].contains(&t.content.index()){
            let _ =destroy(self, world, direction.clone());
        }

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
}
