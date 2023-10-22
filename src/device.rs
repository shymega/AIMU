pub mod trigger;
pub mod vmouse;

#[derive(Debug, Default)]
pub struct Config {
    pub screen: f32,
    // pub orient: [i8; 9],
    pub trigger: Option<trigger::Config>,
}
