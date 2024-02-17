use std::io::Write;
use charting_tools::charted_coordinate::ChartedCoordinate;
use rand::seq::SliceRandom;
use robotics_lib::interface::{Direction, go};
use robotics_lib::runner::Runnable;
use robotics_lib::world::tile::{Content, TileType};
use robotics_lib::world::World;
use crate::robot::{Mode, MyRobot};
use crate::utils::opposite_direction;


pub(crate) fn run_follow_street_mode(robot: &mut MyRobot, world: &mut World){
    let mut view;
    let mut found_street =false;
    let mut directions=[(Direction::Up, (0,1)), (Direction::Right, (1,2)), (Direction::Down, (2,1)), (Direction::Left, (1,0))];

    match go(robot, world, robot.direction.clone()) {
        Ok((v, _)) => { view=v; }
        Err(_) => { return }
    }

    for (dir, index) in directions.iter(){
        if let Some(t)=&view[index.0][index.1]{
            if t.content.index()==7 && t.content.get_value().1.unwrap().start!=t.content.get_value().1.unwrap().end{
                robot.save_tile(dir, &t.content);
                robot.mode=Mode::SearchingContent;
                let _ = robot.file.write_all(format!("Bank saved FollowStreet\n").as_bytes());
                break
            }else if t.content.to_default()==Content::Building || t.content.to_default()==Content::Market(0){
                robot.mode= Mode::ScanBank;
            }
        }
    }
    if let Mode::FollowStreet=robot.mode {
        directions.shuffle(&mut rand::thread_rng());
        for (dir, coord) in directions.iter().filter(|(d, _)| d != &opposite_direction(&robot.direction)) {
            if let Some(t) = &view[coord.0][coord.1] {
                if t.tile_type == TileType::Street {
                    found_street = true;
                    robot.direction = dir.clone();
                    break
                }
            }
        }
        if !found_street {
            robot.rounds = robot.rounds + 1;
            if robot.rounds == 2 {
                robot.chart.save(&Content::JollyBlock(0), &ChartedCoordinate::new(robot.get_coordinate().get_row(), robot.get_coordinate().get_col()));
                robot.mode = Mode::SearchingContent;
            } else {
                robot.direction = opposite_direction(&robot.direction);
            }
        }
    }
}