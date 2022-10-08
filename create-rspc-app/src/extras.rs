use strum::{Display, EnumIter, EnumString};

#[derive(Debug, EnumIter, EnumString, Display)]
pub enum Extras {
    TailwindCSS,
    Tracing,
    None,
}
