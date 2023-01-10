use serde::Deserialize;

/// The rating information of a media item.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rating {
    percentage: u16,
    watching: u32,
    votes: u32,
    loved: u32,
    hated: u32,
}

impl Rating {
    pub fn new(percentage: u16) -> Self {
        Self {
            percentage,
            watching: 0,
            votes: 0,
            loved: 0,
            hated: 0,
        }
    }

    pub fn new_with_metadata(percentage: u16, watching: u32, votes: u32, loved: u32, hated: u32) -> Self {
        Self {
            percentage,
            watching,
            votes,
            loved,
            hated,
        }
    }

    pub fn percentage(&self) -> &u16 {
        &self.percentage
    }

    pub fn watching(&self) -> &u32 {
        &self.watching
    }

    pub fn votes(&self) -> &u32 {
        &self.votes
    }

    pub fn loved(&self) -> &u32 {
        &self.loved
    }

    pub fn hated(&self) -> &u32 {
        &self.hated
    }
}