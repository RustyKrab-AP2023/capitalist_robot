use std::io::Write;
use rand::{seq::SliceRandom, Rng, thread_rng};
use recycle_by_ifrustrati::tool::recycle;
use robotics_lib::interface::{Direction, go, robot_view};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::{Content, TileType};
use robotics_lib::world::World;
use rust_and_furious_dynamo::dynamo::Dynamo;
use crate::robot::{Mode, MyRobot};
use crate::utils::{opposite_direction, quantity};

pub(crate) fn run_searching_content_mode(robot: &mut MyRobot, world: &mut World){
    let mut directions=[(Direction::Up, (0,1)), (Direction::Right, (1,2)), (Direction::Down, (2,1)), (Direction::Left, (1,0))];
    let view= robot_view(robot, world);

    if robot.explorer_pause{
        robot.expl_pause_tick_time= robot.expl_pause_tick_time+1;
    }

    //look at all directions
    for (dir, coord) in directions.iter(){
        if let Some(t)=&view[coord.0][coord.1]{
            robot.check_content(world, &dir.clone(), t);
            //backpack is full
            if quantity(robot.robot.backpack.get_contents())==robot.robot.backpack.get_size() && !robot.explorer_pause{
                match recycle(robot, 0) {
                    Ok(_) => {
                        //coins in the backpack > 70% of backpack size (14)
                        if robot.robot.backpack.get_contents().get(&Content::Coin(0)).unwrap() >= &((robot.robot.backpack.get_size() as f64 * 0.7).round() as usize){
                            robot.mode = Mode::SearchingBank;
                            break
                        }
                    },
                    Err(err) => {
                        if err==LibError::NotEnoughEnergy{
                            *robot.get_energy_mut()=Dynamo::update_energy();
                        }else{
                            if robot.robot.backpack.get_contents().get(&Content::Coin(0)).unwrap() >= &((robot.robot.backpack.get_size() as f64 * 0.7).round() as usize){
                                robot.mode = Mode::SearchingBank;
                                break
                            }
                        }
                    }
                }
            }
        }
    }

    if let Mode::SearchingContent =robot.mode{
        let vertex_search_content = [(&view[0][0], Direction::Up, Direction::Left), (&view[0][2], Direction::Up, Direction::Right), (&view[2][0], Direction::Down, Direction::Left), (&view[2][2], Direction::Down, Direction::Right)];
        let vertex_avoid_content= [(&view[0][0], Direction::Down, Direction::Right), (&view[0][2], Direction::Down, Direction::Left), (&view[2][0], Direction::Up, Direction::Right), (&view[2][2], Direction::Up, Direction::Left)];

        //if the robot doesnt catch the content and uses vertex_search_content it will stuck
        let vertex=
            if robot.explorer_pause{ vertex_avoid_content }
            else{ vertex_search_content };

        //in order to prevent the robot stucks: after a certain time change direction randomly (except the opposite direction and the actual one of the robot)
        if (robot.count_tick%124)==0{
            let _ = robot.file.write_all(format!("Change\n").as_bytes());
            loop {
                directions.shuffle( & mut thread_rng());
                if directions[0].0!=robot.direction &&  directions[0].0!=opposite_direction(&robot.direction){
                    robot.direction=directions[0].0.clone();
                    break
                }
            }
        }

        //check if there is some content on the robot view's vertexes
        for (v, d1, d2) in vertex.iter(){
            if let Some(t)=v{
                if [0, 2, 4, 7, 10].contains(&t.content.index()) && (t.tile_type!=TileType::Street || !robot.avoid_street){ //tolto 1: Tree, 4: Coin
                    if thread_rng().gen::<f64>() < 0.5{
                        robot.direction=d1.clone();
                    }else{
                        robot.direction=d2.clone();
                    }
                }
            }
        }

        //try to go in that direction, otherwise change it
        if let Err(_)=go(robot, world, robot.direction.clone()){
            directions.shuffle(&mut thread_rng());
            for (d, _) in directions.iter(){
                match go(robot, world, d.clone()){
                    Ok(_)=> {
                        robot.direction=d.clone();
                        break
                    },
                    Err(e) => {
                        if e==LibError::OutOfBounds{
                            if let Ok(_)=go(robot, world, opposite_direction(&robot.direction)){
                                robot.direction=opposite_direction(&robot.direction);
                                break
                            }
                        }
                    }
                }
            }
        }
    }
}