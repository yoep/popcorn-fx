use crate::core::players::Player;

pub trait PlayerManager {
    fn active_player(&self) -> Option<&Box<dyn Player>>;

    fn players(&self) -> &Vec<Box<dyn Player>>;
}

#[derive(Debug)]
pub struct DefaultPlayerManager {

}

impl DefaultPlayerManager {

}

impl PlayerManager for DefaultPlayerManager {
    fn active_player(&self) -> Option<&Box<dyn Player>> {
        todo!()
    }

    fn players(&self) -> &Vec<Box<dyn Player>> {
        todo!()
    }
}