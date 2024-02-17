use std::collections::HashMap;
use std::io::Write;
use robotics_lib::world::World;
use robotics_lib::world::tile::Content;
use pmp_collect_all::CollectAll;
use crate::robot::{Mode, CapitalistRobot};

pub(crate) fn run_scan_content_mode(robot: &mut CapitalistRobot, world: &mut World){
    let mut contents=HashMap::new();
    //fill it with the required contents, quantity=0 means it will try to collect all the available content
    contents.insert(Content::Rock(0), 0);
    contents.insert(Content::Fish(0), 0);
    contents.insert(Content::Garbage(0), 0);
    //contents.insert(Content::Tree(0), 0);
    contents.insert(Content::Coin(0), 0);

    //collect all the required items in the range (cross scan)
    //non capisco se funaziona
    CollectAll::collect_items(robot, world, 5, contents);

    robot.mode=Mode::SearchingContent;
}