use std::cell::Cell;
use std::io;
use crate::helpers::*;

use behaviors::basic_ia::BasicIA;
use crate::core::behaviors::CanBuildActions;

use crate::models::{AllData, Cellule};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn main() {

    let initial_cellules = load_cellules();
    let nombre_de_cellules = initial_cellules.len();
    let number_of_bases = load_nombre_de_bases();
    let my_base_index = load_index_base();
    let opp_base_index = load_index_base();

    let my_bot = BasicIA {};

    // game loop
    loop {
        for i in 0..nombre_de_cellules as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let resources = parse_input!(inputs[0], i32); // the current amount of eggs/crystals on this cell
            let my_ants = parse_input!(inputs[1], i32); // the amount of your ants on this cell
            let opp_ants = parse_input!(inputs[2], i32); // the amount of opponent ants on this cell
        }

        // WAIT | LINE <sourceIdx> <targetIdx> <strength> | BEACON <cellIdx> <strength> | MESSAGE <text>

        let all_data = AllData {
            cellules: initial_cellules.clone(),
            my_base_index,
            opp_base_index
        };

        my_bot.execute_actions(&all_data);
        // fixme voir pourquoi on a un warning dans la console println!("MESSAGE hello world");
    }
}

mod behaviors {

    pub mod basic_ia {
        use crate::core::behaviors::CanBuildActions;
        use crate::models::AllData;

        pub struct BasicIA;

        impl CanBuildActions for BasicIA {
            fn build_actions(&self, all_data: &AllData) -> Vec<String> {
                vec![
                    "WAIT".to_string()
                ]
            }
        }
    }
}

mod core {
    pub mod behaviors {
        use crate::models::AllData;

        pub trait CanBuildActions {
            fn build_actions(&self, all_data: &AllData) -> Vec<String>;

            fn execute_actions(&self, all_data: &AllData) {
                self.build_actions(all_data)
                    .into_iter()
                    .for_each(|action| println!("{}", action))
            }
        }

    }
}

mod models {

    pub struct AllData {
        pub cellules: Vec<Cellule>,
        pub my_base_index: i32,
        pub opp_base_index: i32,
    }

    #[derive(Clone)]
    pub struct Cellule {
        pub r#type: i32,
        pub identifiant: i32,
        pub nombre_de_crystal: i32,
        pub nombre_insectes: Option<i32>
    }
}

mod helpers {


    trait CanSort<T> {
        fn sort_immut(&self) -> T;
    }

    pub mod vec {
        use crate::helpers::CanSort;
        use crate::models::Cellule;

        impl CanSort<Vec<Cellule>> for Vec<Cellule> {
            fn sort_immut(&self) -> Vec<Cellule> {
                let mut cloned = self.clone();
                cloned.sort_by(|cellule1, cellule2| {
                    cellule1.nombre_de_crystal.cmp(&cellule2.nombre_de_crystal)
                });
                cloned.into()
            }
        }
    }

    use std::io;
    use crate::models::Cellule;

    pub fn load_index_base() -> i32 {
        let mut inputs = String::new();
        io::stdin().read_line(&mut inputs).unwrap();
        inputs.split_whitespace()
            .map(|index_str| parse_input!(index_str, i32) as i32)
            .collect::<Vec<_>>()
            .first()
            .map(|res| res.clone())
            .unwrap_or_else(|| {
                eprintln!("erreur du chargement de l'index de la base");
                0
            })
    }

    pub fn load_nombre_de_bases() -> i32 {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        parse_input!(input_line, i32)
    }

    pub fn load_cellules() -> Vec<Cellule> {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let number_of_cells = parse_input!(input_line, i32); // amount of hexagonal cells in this map

        (0..number_of_cells)
            .into_iter()
            .map(|index| {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();
                let inputs = input_line.split(" ").collect::<Vec<_>>();
                let _type = parse_input!(inputs[0], i32); // 0 for empty, 1 for eggs, 2 for crystal
                let initial_resources = parse_input!(inputs[1], i32); // the initial amount of eggs/crystals on this cell
                let neigh_0 = parse_input!(inputs[2], i32); // the index of the neighbouring cell for each direction
                let neigh_1 = parse_input!(inputs[3], i32);
                let neigh_2 = parse_input!(inputs[4], i32);
                let neigh_3 = parse_input!(inputs[5], i32);
                let neigh_4 = parse_input!(inputs[6], i32);
                let neigh_5 = parse_input!(inputs[7], i32);

                Cellule {
                    r#type: _type,
                    identifiant: index,
                    nombre_de_crystal: initial_resources,
                    nombre_insectes: None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn update_cellules(initial_cellules: &Vec<Cellule>) -> Vec<Cellule> {
        // todo update
        vec![]
    }
}
