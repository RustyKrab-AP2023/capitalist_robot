use std::cmp::min;
use std::collections::HashMap;
use std::ops::Range;
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_map::{ChartedMap, SavedQuantity};
use rand::{seq::SliceRandom, thread_rng};
use robotics_lib::interface::{Direction, discover_tiles, go, put, robot_map, robot_view, teleport};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::{coordinates::Coordinate, World};
use robotics_lib::world::tile::{Tile, Content, TileType};
use rust_and_furious_dynamo::dynamo::Dynamo;
use rustici_planner::tool::Action;
use crate::CapitalistRobot;


///it discovers tile not discovered yet to save energy
pub(crate) fn smart_discovery(robot: &mut impl Runnable, world: &mut World, tiles: &[(usize, usize)]) -> Result<HashMap<(usize, usize), Option<Tile>>, LibError> {
    let mut to_discover= Vec::new();
    let map = robot_map(world).unwrap();
    for (x,y) in tiles.iter(){
        if let None=map[*x][*y]{
            to_discover.push((*x,*y))
        }
    }
    discover_tiles(robot, world, &to_discover[..])
}

///return the direction to a non Street and walkable tale if it finds it
pub(crate) fn look_around(robot: &impl Runnable, world: &mut World) -> Option<Direction>{
    let mut res=None;
    let directions=[(Direction::Up, (0,1)), (Direction::Right, (1,2)), (Direction::Down, (2,1)), (Direction::Left, (1,0))];
    let view=robot_view(robot, world);
    for (dir, coord) in directions.iter(){
        if let Some(t)=&view[coord.0][coord.1]{
            if t.tile_type!=TileType::Street && t.tile_type.properties().walk(){
                res=Some(dir.clone());
                break
            }
        }
    }
    res
}

///robot goes in whatever tile he can go
pub(crate) fn go_where_you_can(robot: &mut impl Runnable, world: &mut World){
    let mut directions=[Direction::Up, Direction::Right, Direction::Down, Direction::Left];
    directions.shuffle(&mut thread_rng());
    for dir in directions.iter(){
        if let Ok(_)=go(robot, world, dir.clone()){
            break
        }
    }
}


///return the coordinate of a tile given the direction and the robot position
pub(crate) fn get_coords_row_col(robot: &impl Runnable, direction: &Direction) -> (usize, usize) {
    let robot_row = robot.get_coordinate().get_row();
    let robot_col = robot.get_coordinate().get_col();
    match direction {
        | Direction::Up => (robot_row - 1, robot_col),
        | Direction::Down => (robot_row + 1, robot_col),
        | Direction::Left => (robot_row, robot_col - 1),
        | Direction::Right => (robot_row, robot_col + 1),
    }
}
pub(crate) fn manhattan_distance(p1: (usize, usize), p2: (usize, usize)) -> usize{
    p1.0.abs_diff(p2.0) + p1.1.abs_diff(p2.1)
}

///check if all the elements in option_list are enough distant from the robot
pub(crate) fn check_distance(option_list: Option<&Vec<(ChartedCoordinate, SavedQuantity)>>, rob_coord: &Coordinate) -> bool{
    let distance= 30;
    if let Some(list)=option_list{
        for (bank_coord,_) in list.iter(){
            if manhattan_distance((bank_coord.get_row(), bank_coord.get_col()), (rob_coord.get_row(), rob_coord.get_col())) < distance{
                return false
            }
        }
    }
    true
}

///check if the robot is enough distant from all the content it has saved
pub(crate) fn enough_distant(rob_coord: &Coordinate, map: &ChartedMap<Content>) -> bool{
    let banks=map.get(&Content::Bank(0..0));
    let closed_streets= map.get(&Content::JollyBlock(0));

    check_distance(banks, rob_coord)  && check_distance(closed_streets, rob_coord)
}

pub(crate) fn opposite_direction(dir: &Direction) -> Direction{
    match dir {
        Direction::Up => {Direction::Down}
        Direction::Down => {Direction::Up}
        Direction::Left => {Direction::Right}
        Direction::Right => {Direction::Left}
    }
}

pub(crate) fn quantity(hm: &HashMap<Content, usize>) -> usize{
    let mut res=0;

    for (_,s ) in hm.iter(){
        res=res+ s;
    }
    res
}

///check if range is valid otherwise change it
pub(crate) fn check_range(range: &mut usize, center:(usize, usize), world_size: usize){
    if center.0 < *range || center.0+*range > world_size || center.1 < *range || center.1+*range > world_size{
        *range= min(min(center.0, world_size-center.0), min(center.1, world_size-center.1));
    }
}

///move the robot following a given vector of Action until it reaches a bank
pub(crate) fn follow_path(robot: &mut CapitalistRobot, path: Vec<Action>, world: &mut World, bank_coord: &ChartedCoordinate, bank_direction: &mut Direction){
    for action in path.iter(){
        if robot.get_energy().get_energy_level() < 300{
            *robot.get_energy_mut()=Dynamo::update_energy();
        }
        match action {
            Action::Move(direction) => {
                if get_coords_row_col(robot, direction)==(bank_coord.0, bank_coord.1){
                    *bank_direction=direction.clone();
                    break
                }else{
                    go(robot, world, direction.clone()).unwrap();
                }
            }
            Action::Teleport(teleport_coord) => { teleport(robot, world, *teleport_coord).unwrap(); }
        }
    }
}

///put all the coins until the backpack is empty or the bank is full
pub(crate) fn deposit_coins(robot: &mut CapitalistRobot, world: &mut World, bank_range: &mut Range<usize>, bank_direction: &Direction){
    let mut coins_in_backpack= *robot.get_backpack().get_contents().get(&Content::Coin(0)).unwrap_or(&0);
    let mut is_full = false;
    let mut no_more_coins = false;
    let mut errors = false;

    while !is_full && !no_more_coins && !errors {
        match put(robot, world, Content::Coin(0), coins_in_backpack, bank_direction.clone()) {
            Ok(deposited_coins) => {
                coins_in_backpack= coins_in_backpack-deposited_coins;
                bank_range.start=bank_range.start+deposited_coins;
                if deposited_coins == 0 { is_full = true; }
            }
            Err(err) => {
                match err {
                    LibError::NoContent => { no_more_coins = true; }
                    LibError::NotEnoughEnergy => { *robot.get_energy_mut()=Dynamo::update_energy(); }
                    _ => {
                        errors = true;
                    }
                }
            }
        }
    }
}