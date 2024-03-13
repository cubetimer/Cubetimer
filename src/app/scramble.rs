use rand::Rng;
use std::fmt::Display;

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum Cubes {
    ThreeByThree,
    TwoByTwo,
    OneByOne,
    FourByFour,
    FiveByFive,
}

impl Display for Cubes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Cubes::OneByOne => "1x1",
            Cubes::TwoByTwo => "2x2",
            Cubes::ThreeByThree => "3x3",
            Cubes::FourByFour => "4x4",
            Cubes::FiveByFive => "5x5",
        };
        write!(f, "{}", string.to_string())
    }
}

#[allow(dead_code)]
pub struct Scrambler {
    cube: Cubes,
}

impl Default for Scrambler {
    fn default() -> Self {
        Self {
            cube: Cubes::ThreeByThree,
        }
    }
}

impl From<Cubes> for Scrambler {
    fn from(value: Cubes) -> Self {
        Self { cube: value }
    }
}

#[allow(dead_code, unused_assignments)]
impl Scrambler {
    pub fn scramble(&self) -> String {
        let length: i8 = match self.cube {
            Cubes::OneByOne => 8,
            Cubes::TwoByTwo => 12,
            Cubes::ThreeByThree => 25,
            Cubes::FourByFour => 40,
            Cubes::FiveByFive => 45,
        };
        let mut options: Vec<String> = vec![];
        if self.cube != Cubes::OneByOne {
            options = vec![
                "R".to_string(),
                "R2".to_string(),
                "R'".to_string(),
                "R".to_string(),
                "U".to_string(),
                "U'".to_string(),
                "U2".to_string(),
                "F".to_string(),
                "F'".to_string(),
                "F2".to_string(),
                "D".to_string(),
                "D'".to_string(),
                "D2".to_string(),
                "L".to_string(),
                "L2".to_string(),
                "L'".to_string(),
            ];
        } else {
            options = [
                "X".to_string(),
                "X'".to_string(),
                "X2".to_string(),
                "Y".to_string(),
                "Y'".to_string(),
                "Y2".to_string(),
                "Z".to_string(),
                "Z'".to_string(),
                "Z2".to_string(),
            ]
            .to_vec();
        }
        if self.cube == Cubes::FourByFour || self.cube == Cubes::FiveByFive {
            options.push("Rw".to_string());
            options.push("Rw2".to_string());
            options.push("Rw".to_string());
            options.push("Rw2".to_string());
            options.push("Rw'".to_string());
            options.push("Lw".to_string());
            options.push("2Lw".to_string());
            options.push("Lw'".to_string());
            options.push("Fw".to_string());
            options.push("Fw2".to_string());
            options.push("Fw'".to_string());
            options.push("Dw".to_string());
            options.push("Dw2".to_string());
            options.push("D2".to_string());
        }
        let mut back = "".to_string();
        let mut scramble: Vec<String> = vec![];
        let mut rng = rand::thread_rng();
        for _i in 1..=length {
            loop {
                let option = &options[rng.gen_range(1..options.len())];
                let mut characters: Vec<char> = option.chars().collect();
                if characters[0].is_digit(10) {
                    characters.remove(0);
                }
                let character = match characters[0].to_string().as_str() {
                    "R" => "R",
                    "R'" => "R",
                    "R2" => "R",
                    "U" => "U",
                    "U'" => "U",
                    "U2" => "U",
                    "F" => "F",
                    "F'" => "F",
                    "F2" => "F",
                    "D" => "D",
                    "D'" => "D",
                    "D2" => "D",
                    "L" => "L",
                    "L'" => "L",
                    "L2" => "L",
                    _ => continue,
                };
                if back == character.to_string() {
                    continue;
                }
                scramble.push(option.to_string());
                back = character.to_string();
                break;
            }
        }
        scramble.join(" ").to_string()
    }
}
