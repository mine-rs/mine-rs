use crate::Result;
use std::{fmt::Debug, marker::PhantomData};

use super::enchant::Enchant;

pub trait Item: Sized + Debug {
    fn id(self) -> u16;
    fn from_id(id: u16) -> Option<Self>;
    fn name(self) -> &'static str;
    fn from_name(name: &str) -> Option<Self>;
    fn display_name(self) -> &'static str;
    fn stack_size(self) -> u8;
    fn max_durability(self) -> Option<u16>;
}

#[derive(Debug, Clone, Copy)]
pub struct Itemstack<T: Item, U: Enchant> {
    pub item: T,
    pub count: i8,
    pub meta: Option<ItemStackMetaData<U>>,
}

impl<T: Item, U: Enchant> Itemstack<T, U> {
    /// Returns a new `ItemStack`
    pub fn new(item: T, count: i8) -> Result<Self> {
        Ok(Self {
            item,
            count,
            meta: None,
        })
    }
}

//TODO: Implement this for real
#[derive(Debug, Clone, Copy)]
pub struct ItemStackMetaData<T> {
    _marker: PhantomData<T>,
}
