use rip_worldgenerator::MyWorldGen;
use robotics_lib::runner::Runner;
use shared_state::SharedStateWrapper;
use capitalist_robot::MyRobot;

fn main() {
    let mut generator= MyWorldGen::new();

    let robot=MyRobot::new(SharedStateWrapper::new());

    match  Runner::new( Box::new( robot), &mut generator){
        Ok(mut runner)=>{
            loop {
                let _= runner.get_robot();
                let _ = runner.game_tick();
            }
        }
        Err(err) => {println!("{:?}", err)}
    }
}