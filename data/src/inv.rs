pub mod enchant;
pub mod item;

#[derive(Debug, Clone, Copy)]
pub enum Slot<I: item::Item, E: enchant::Enchant> {
    Empty,
    //TODO: Come up with a better name than "Filled" as that implies a full stack
    Filled(item::Itemstack<I, E>),
}
