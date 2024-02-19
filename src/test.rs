use std::cmp::min;
use std::collections::HashMap;
use robotics_lib::interface::{Direction};
use robotics_lib::world::tile::Content;
use crate::utils::{check_range, opposite_direction, quantity};

#[test]
fn opposite_direction_test() {
    assert_eq!(opposite_direction(&Direction::Right), Direction::Left);
    assert_eq!(opposite_direction(&Direction::Left), Direction::Right);
    assert_eq!(opposite_direction(&Direction::Up), Direction::Down);
    assert_eq!(opposite_direction(&Direction::Down), Direction::Up);
}

#[test]
fn quantity_test(){
    let mut content_list= HashMap::<Content, usize>::new();
    content_list.insert(Content::Coin(0), 1);
    content_list.insert(Content::Rock(0), 2);
    content_list.insert(Content::Garbage(0), 3);
    assert_eq!(quantity(&content_list),  6);
}

#[test]
fn check_range_test_valid_range(){
    let mut range=1;
    let old_range=range;
    let center= (3,3);
    let world_size=5;
    check_range(&mut range, center, world_size);
    assert_eq!(range, old_range);
}

fn check_range_test_invalid_range(){
    let mut range=5;
    let center=(1,3);
    let world_size=5;
    check_range(&mut range, center, world_size);
    assert_eq!(range, min(min(center.0, world_size-center.0), min(center.1, world_size-center.1)));
}