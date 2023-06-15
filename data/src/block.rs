pub trait Block {
    fn id() -> u16;
    fn display_name() -> &'static str;
    fn name() -> &'static str;
    fn hardness() -> f32;
    fn stack_size() -> u8;
    fn diggable() -> bool;
    fn bounding_box() -> (); //TODO: add return value
    fn drops() -> (); //TODO: add return value
    fn transparent() -> bool;
    fn emit_ligth() -> u8;
    fn filter_light() -> u8;
    fn material() -> Option<&'static str>;
    fn harvest_tools() -> Option<()>; //TODO: add return value
    fn variations() -> Option<()>; //TODO: add return value
    fn states() -> Option<()>; //TODO: add return value
    fn min_state_id() -> Option<u16>;
    fn max_state_id() -> Option<u16>;
    fn default_state() -> Option<u16>;
    fn resistance() -> Option<f32>;
}
