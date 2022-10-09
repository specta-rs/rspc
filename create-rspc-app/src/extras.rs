use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, EnumIter, EnumString, Display)]
pub enum Extras {
    TailwindCSS,
    Tracing,
    None,
}
