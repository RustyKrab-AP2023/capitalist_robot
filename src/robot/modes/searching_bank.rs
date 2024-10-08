use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_map::SavedQuantity;
use robotics_lib::interface::{Direction, robot_map};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;
use rustici_planner::tool::{Destination, Planner, PlannerError, PlannerResult};
use crate::robot::{Mode, CapitalistRobot};
use crate::utils::{check_range, deposit_coins, follow_path, manhattan_distance};

pub(crate) fn run_searching_bank_mode(robot: &mut CapitalistRobot, world: &mut World){
    let mut min_distance =usize::MAX;
    let mut current_distance ;
    let robot_coord =ChartedCoordinate::new(robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
    let mut bank_coord= robot_coord;
    let mut bank_range=0..0;
    let mut banks_saved= false;
    let mut path_found=false;

    if let Some(bank_list)=robot.chart.get(&Content::Bank(0..0)){
        banks_saved= true;
        let mut to_remove=Vec::new();

        //find the closest bank and check if a bank is full
        for (coord, saved_quantity) in bank_list.iter(){
            if let SavedQuantity::ContentRange(range)=saved_quantity{
                if range.start!=range.end{
                    current_distance=manhattan_distance((robot.get_coordinate().get_row(), robot.get_coordinate().get_col()), (coord.get_row(), coord.get_col()));
                    if current_distance < min_distance{
                        min_distance= current_distance;
                        bank_coord=*coord;
                        bank_range=range.clone();
                    }
                }else{
                    to_remove.push(*coord);
                }
            }
        }

        //remove full banks and replace them with JollyBlock to let avoid_street work
        for coord in to_remove.iter(){
            let _ =robot.chart.remove(&Content::Bank(0..0), *coord);
            robot.chart.save(&Content::JollyBlock(0), coord);
        }


        //if found a not full bank
        if bank_coord!=robot_coord{
            path_found=true;
            let mut bank_direction=Direction::Right;
            let destination= Destination::go_to_coordinate((bank_coord.get_row(), bank_coord.get_col()));
            match Planner::planner(robot, destination, world){
                Ok(res) => {
                    match res {
                        PlannerResult::Path((path, _)) => {
                            follow_path(robot, path, world, &bank_coord, &mut bank_direction);

                            deposit_coins(robot, world, &mut bank_range, &bank_direction);

                            //update saved bank range, if full replace it with a JollyBlock to let avoid_street work
                            let _ = robot.chart.remove(&Content::Bank(0..0), bank_coord);
                            if bank_range.start==bank_range.end{
                                robot.chart.save(&Content::JollyBlock(0), &bank_coord);
                            }else{
                                robot.chart.save(&Content::Bank(bank_range.start..bank_range.end), &bank_coord);
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    robot.explorer_pause=true;
                }
            }
        }
    }
    if !banks_saved || !path_found{

        let mut range=30;
        let center= (robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
        let old_robot_coord= center;
        let explorer_destination = Destination::explore(robot.get_energy().get_energy_level(), range);
        match Planner::planner(robot, explorer_destination, world) {
            Ok(res) => {
                match res {
                    PlannerResult::RadiusExplored => {
                        //if the robot did not move
                        if (robot.get_coordinate().get_row(), robot.get_coordinate().get_col())==old_robot_coord{
                            robot.mode=Mode::SearchingContent;
                            robot.explorer_pause=true;
                        }else{
                            let map= robot_map(world).unwrap();
                            let world_size= map.len();
                            check_range(&mut range, center, world_size);
                            for row_index in center.0-range..center.1+range{
                                for col_index in center.1-range..center.1+range{
                                    if let Some(tile)=&map[row_index][col_index]{
                                        if tile.content.index()==7 && tile.content.get_value().1.unwrap().start!=tile.content.get_value().1.unwrap().end{
                                            robot.chart.save(&tile.content, &ChartedCoordinate(row_index, col_index));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PlannerResult::MapAllExplored => {robot.terminated=true;}
                    _ => {}
                }
            }
            Err(err) => {
                match err {
                    PlannerError::MaxEnergyReached => {*robot.get_energy_mut()=Dynamo::update_energy();}
                    PlannerError::RoboticLibError(lib_err) => {
                        if let LibError::NotEnoughEnergy=lib_err{
                            *robot.get_energy_mut()=Dynamo::update_energy();
                        }
                    }
                    _ => {
                        robot.mode=Mode::SearchingContent;
                        robot.explorer_pause=true;
                    }
                }
            }
        }
        //if the robot discovered more than 50% of the world's tile terminate it
        if world.get_discoverable() > ((robot_map(world).unwrap().len() as f64) * 0.5) as usize{
            robot.terminated=true;
        }
    }else{
        robot.mode=Mode::SearchingContent;
    }

}