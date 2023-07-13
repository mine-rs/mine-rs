pub mod enchant;
pub mod item;

#[derive(Debug, Clone, Copy)]
pub enum Slot<I: item::Item, E: enchant::Enchant> {
    Empty,
    Populated(item::Itemstack<I, E>),
}
