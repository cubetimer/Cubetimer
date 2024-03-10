use std::fmt::Display;

#[allow(dead_code)]
pub enum Cubes {
    ThreeByThree,
    TwoByTwo,
    OneByOne,
    FourByFour,
}

impl Display for Cubes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Cubes::OneByOne => "1x1",
            Cubes::TwoByTwo => "2x2",
            Cubes::ThreeByThree => "3x3",
            Cubes::FourByFour => "4x4",
        };
        write!(f, "{}", string.to_string())
    }
}

#[allow(dead_code)]
pub struct Scrambler {
    cube: Cubes,
}

#[allow(dead_code)]
impl Scrambler {
    fn scramble(&self) {}
}
