use std::convert::TryInto;

use bevy::prelude::*;

#[derive(Clone,Component,Reflect, Default, Copy, Debug, PartialEq)]
pub enum Direction {
    UpRight,
    Right,
    DownRight,
    DownLeft,
    Left,
    UpLeft,
    #[default]
    None
}

#[derive(Component)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component)]
pub struct Following(pub Entity);

#[derive(Component)]
pub struct Follower(pub Entity);

#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct Hex {
    pub q: f32,
    pub r: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct HexHistory(pub Vec<Hex>);

#[derive(Component, Reflect, Default)]
pub struct Head {
    pub direction: Direction,
    pub last_direction: Direction
}

#[derive(Component, Reflect, Default)]
pub struct Body(pub Vec<Entity>);

#[derive(Component, Reflect, Default)]
pub struct Tail;

#[derive(Component, Reflect, Default)]
pub struct Crumple;

#[derive(Component, Reflect, Default)]
pub struct Segment;

#[derive(Component, Reflect, Default)]
pub struct MovementCooldown {
    pub timer: Timer,
}


#[derive(Default, Reflect, Hash, Component, Clone, Copy)]
#[reflect(Hash)]
pub struct Pcg32RandomT {
    state: u64,
    inc: u64
}

impl Pcg32RandomT {    
    
    pub fn new(initstate: u64, initseq: u64) -> Self {
        Self{state: initstate, inc: (initseq << 1u64) | 1u64,}
    }

    pub fn pcg32_random_r(&mut self) -> u32 {
        let oldstate = self.state;
        // Advance internal state
        self.state = oldstate * 6364136223846793005u64 + self.inc;
        // Calculate output function (XSH RR), uses old state for max ILP
        let xorshifted = ((oldstate >> 18) ^ oldstate >> 27) as u32;
        let rot = oldstate >> 59;
        return (xorshifted >> rot) | (xorshifted << ((rot.wrapping_neg()) & 31));
    }

    pub fn bounded_rand(&mut self, range: u32) -> u32 {
        let mut x = self.pcg32_random_r();
        let mut m = x as u64 * range as u64;
        let mut l = m as u32;
        if l < range {
            let mut t = range.wrapping_neg();
            if t >= range {
                t -= range;
                if t >= range {
                    t %= range;
                }
            }
            while l < t {
                x = self.pcg32_random_r();
                m = x as u64 * range as u64;
                l = m as u32;
            }
        }
        return (m >> 32) as u32;
    }


}