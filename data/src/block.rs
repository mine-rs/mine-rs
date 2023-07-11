pub trait Block: Copy {
    fn id(self) -> u16;
    fn from_id(id: u16) -> Option<Self>;

    fn display_name(self) -> &'static str;

    fn name(self) -> &'static str;
    fn from_name(name: &str) -> Option<Self>;

    fn hardness(self) -> Option<f64>;
    fn stack_size(self) -> u8;
    fn diggable(self) -> bool;
    fn bounding_box(self) -> (); //TODO: add return value
    fn drops(self) -> (); //TODO: add return value
    fn transparent(self) -> bool;
    fn emit_light(self) -> u8;
    fn filter_light(self) -> u8;
    fn material(self) -> Option<&'static str>;
    fn harvest_tools(self) -> Option<()>; //TODO: add return value
    fn variations(self) -> Option<()>; //TODO: add return value
    fn states(self) -> Option<()>; //TODO: add return value
    fn min_state_id(self) -> Option<u16>;
    fn max_state_id(self) -> Option<u16>;
    fn default_state(self) -> Option<u16>;
    fn resistance(self) -> Option<f32>;
}
