use crate::state::{Input, State};

pub trait Ui: Sized {
    type Error;
    fn on_panic();
    fn setup() -> Result<Self, Self::Error>;
    fn render(&mut self, app: &State) -> Result<(), Self::Error>;
    fn get_input(&mut self) -> Result<Input, Self::Error>;
    fn finish(&mut self) -> Result<(), Self::Error>;
}
