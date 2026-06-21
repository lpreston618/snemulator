use serde::Serialize;

#[derive(Clone, Copy, Default, Debug, Serialize)]
pub enum Direction {
    #[default]
    AtoB,
    BtoA,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub enum AddressIncMode {
    #[default]
    Inc,
    Fixed,
    Dec,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub enum TransferPattern {
    #[default]
    Pattern0,
    Pattern1,
    Pattern2,
    Pattern3,
    Pattern4,
    Pattern5,
    Pattern6,
    Pattern7,
}