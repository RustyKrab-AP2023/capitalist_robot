use charting_tools::charted_coordinate::ChartedCoordinate;
use robotics_lib::interface::robot_map;
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;
use crate::robot::{Mode, CapitalistRobot};
use crate::utils::{check_range, smart_discovery};

pub(crate) fn run_scan_bank_mode(robot: &mut CapitalistRobot, world: &mut World){
    let mut range=9;
    let center=(robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
    let mut coordinates=Vec::new();
    let mut bank_found= false;


    let world_size= robot_map(world).unwrap().len();
    check_range(&mut range, center, world_size);
    for i in center.0-range..center.0+range{
        for j in center.1-range..center.1+range{
            coordinates.push((i,j));
        }
    }

    //it discovers tile not discovered yet to save energy
    match smart_discovery(robot,world, &coordinates[..]) {
        Ok(view) => {
            for (coord, tile) in view.iter(){
                if let Some(t)=tile{
                    if t.content.index()==7 && t.content.get_value().1.unwrap().start!=t.content.get_value().1.unwrap().end{
                        robot.chart.save(&t.content, &ChartedCoordinate::new(coord.0, coord.1));
                        bank_found= true;
                    }
                }
            }
            //save a tile in order to set avoid_street to true next tick
            if !bank_found{
                robot.chart.save(&Content::JollyBlock(0), &ChartedCoordinate::new(robot.robot.coordinate.get_row(), robot.robot.coordinate.get_col()));
            }
            robot.sblock(world);
            robot.mode=Mode::SearchingContent;
        }
        Err(e) => {
            match e {
                LibError::NotEnoughEnergy => {
                    println!("Not enough energy");
                    *robot.get_energy_mut()=Dynamo::update_energy();
                }
                LibError::NoMoreDiscovery => {
                    println!("No more discovery");
                    robot.sblock(world);
                    robot.mode=Mode::SearchingContent;
                    robot.no_more_discovery=true;
                }
                _ => {println!("error");}
            }
        }
    }
}