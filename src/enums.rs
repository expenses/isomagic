use std::str::FromStr;

#[derive(PartialEq)]
pub enum Side {
    Top,
    Front,
    Left,
    Right,
    Back,
    Bottom
}

impl FromStr for Side {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(match string {
            "top"    => Side::Top,
            "front"  => Side::Front,
            "left"   => Side::Left,
            "right"  => Side::Right,
            "back"   => Side::Back,
            "bottom" => Side::Bottom,
            _ => return Err(format!("No corresponding side for '{}'", string))
        })
    }
}

impl Side {
    pub fn to_str(&self) -> &'static str {
        match *self {
            Side::Top   => "top",
            Side::Front => "front",
            Side::Left  => "left",
            Side::Right => "right",
            Side::Back  => "back",
            Side::Bottom => "bottom"
        }
    }

    pub fn all() -> Vec<Side> {
        vec![Side::Top, Side::Front, Side::Left, Side::Right, Side::Back, Side::Bottom]
    }
}

#[derive(PartialEq)]
pub enum View {
    Face,
    FourtyFive,
    FourtyFiveIso,
    TwentyTwoPointFive,
    TwentyTwoPointFiveIso
}

impl FromStr for View {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(match string {
            "face" => View::Face,
            "45" => View::FourtyFive,
            "45 iso" => View::FourtyFiveIso,
            "22.5" => View::TwentyTwoPointFive,
            "22.5 iso" => View::TwentyTwoPointFiveIso,
            _ => return Err(format!("No corresponding view for '{}", string))
        })
    }
}

impl View {
    pub fn to_str(&self) -> &'static str {
        match *self {
            View::Face => "face",
            View::FourtyFive => "45",
            View::FourtyFiveIso => "45_iso",
            View::TwentyTwoPointFive => "22.5",
            View::TwentyTwoPointFiveIso => "22.5_iso"
        }
    }

    pub fn all() -> Vec<View> {
        vec![View::Face, View::FourtyFive, View::FourtyFiveIso, View::TwentyTwoPointFive, View::TwentyTwoPointFiveIso]
    }
}
