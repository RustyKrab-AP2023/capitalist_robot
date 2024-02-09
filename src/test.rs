use rip_worldgenerator::MyWorldGen;
use robotics_lib::interface::discover_tiles;
use robotics_lib::world::World;
use robotics_lib::world::world_generator::Generator;
use crate::robot::*;
use crate::utils::*;

#[test]
fn smart_discovery_test(){
    let mut robot= MyRobot::new();
    let mut generator = MyWorldGen::new();
    let (world, _, _, _, _)=generator.gen();



    //3x3 matrix of tile
    let to_discover=[(0,0), (0,1), (0,2), (1,0), (1,1), (1,2), (2,0), (2,1), (2,2)];
    let discovered= [(0,0), (0,1), (0,2)];
    discover_tiles(&mut robot, World, )
}