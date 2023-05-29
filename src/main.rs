use std::cell::Cell;
use std::io;
use crate::helpers::*;

use behaviors::basic_ia::BasicIA;
use crate::behaviors::basic_ia_with_eggs_first::BasicIAWithEggsFirst;
use crate::behaviors::basic_ia_with_path_finder::BasicIAWithPathFinder;
use crate::core::behaviors::CanBuildActions;

use crate::models::{AllData, Cellule};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn main() {

    let initial_cellules = load_cellules();
    let _ = load_nombre_de_bases();
    let my_base_index = load_index_base();
    let opp_base_index = load_index_base();

    let mut my_bot = BasicIAWithPathFinder :: new();

    let mut index_tour = 0;

    // game loop
    loop {
        index_tour += 1;

        // WAIT | LINE <sourceIdx> <targetIdx> <strength> | BEACON <cellIdx> <strength> | MESSAGE <text>
        let updated_cellules = update_cellules(&initial_cellules);

        let all_data = AllData {
            cellules: updated_cellules,
            my_base_index,
            opp_base_index,
            tour_actuel: index_tour
        };

        my_bot.execute_actions(&all_data);
    }
}

mod behaviors {

    /*
    algo basic qui met une LINE entre la base allié et la meilleure cellule
     */
    pub mod basic_ia {
        use crate::core::behaviors::CanBuildActions;
        use crate::helpers::CanSort;
        use crate::models::{AllData, Cellule};

        pub struct BasicIA {
            pub current_target: Option<i32>
        }

        impl CanBuildActions for BasicIA {
            fn build_actions(&mut self, all_data: &AllData) -> Vec<String> {
                let sorted_cellules = all_data
                    .cellules.clone()
                    .into_iter()
                    .filter(|cellule| cellule.nombre_de_crystal > 0)
                    .collect::<Vec<_>>()
                    .sort_immut();

                self.update_target(&sorted_cellules);

                let poid = 1;
                let action = format!(
                    "LINE {} {} {}",
                    all_data.my_base_index,
                    self.current_target.unwrap_or(1),
                    poid
                );

                vec![
                    action
                ]
            }
        }

        impl BasicIA {
            fn update_target(&mut self, cellules_sorted: &Vec<Cellule>) {
                let target_exist = cellules_sorted.iter()
                    .find(|cellule| cellule.identifiant == self.current_target.unwrap_or(-1))
                    .is_some();

                if (!target_exist) {
                    self.current_target = cellules_sorted.first().map(|cellule| cellule.identifiant)
                }
            }
        }
    }

    /*
    algo basic qui met une LINE entre la base allié et la meilleure cellule
     */
    pub mod basic_ia_with_eggs_first {
        use crate::core::behaviors::CanBuildActions;
        use crate::core::path_finders::CanGiveBestTarget;
        use crate::helpers::CanSort;
        use crate::models::{AllData, Cellule};

        pub struct BasicIAWithEggsFirst {
            pub current_target: Option<i32>
        }

        impl CanBuildActions for BasicIAWithEggsFirst {
            fn build_actions(&mut self, all_data: &AllData) -> Vec<String> {

                let sorted_cellules_by_crystal = all_data
                    .cellules.clone()
                    .into_iter()
                    .filter(|cellule| cellule.nombre_de_crystal > 0 && cellule.r#type == 2)
                    .collect::<Vec<_>>()
                    .sort_immut();

                let sorted_cellules_by_eggs = all_data
                    .cellules.clone()
                    .into_iter()
                    .filter(|cellule| cellule.nombre_de_crystal > 0 && cellule.r#type == 1)
                    .collect::<Vec<_>>()
                    .sort_immut();

                if all_data.tour_actuel < 7 {

                    let best_eggs = sorted_cellules_by_eggs.first().unwrap();

                    let action = format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        best_eggs.identifiant,
                        1
                    );
                    vec![action]
                } else {
                    sorted_cellules_by_crystal
                        .into_iter()
                        .map(|cellule| {
                            let poid = 2;
                            format!(
                                "LINE {} {} {}",
                                all_data.my_base_index,
                                cellule.identifiant,
                                poid
                            )
                        }).collect::<Vec<_>>()
                }
            }
        }

        impl BasicIAWithEggsFirst {
            pub fn new() -> Self {
                Self {
                    current_target: None
                }
            }

            fn update_target(&mut self, cellules_sorted: &Vec<Cellule>) {
                let target_exist = cellules_sorted.iter()
                    .find(|cellule| cellule.identifiant == self.current_target.unwrap_or(-1))
                    .is_some();

                if (!target_exist) {
                    self.current_target = cellules_sorted.first().map(|cellule| cellule.identifiant)
                }
            }
        }
    }

    pub mod basic_ia_with_path_finder {
        use crate::core::behaviors::CanBuildActions;
        use crate::core::path_finders::CanGiveBestTarget;
        use crate::helpers::CanSort;
        use crate::models::{AllData, Cellule};
        use crate::path_finders::brutal::BrutalPathFinder;

        pub struct BasicIAWithPathFinder {
            pub current_target: Option<i32>,
            path_finder: Box<dyn CanGiveBestTarget>
        }

        impl CanBuildActions for BasicIAWithPathFinder {
            fn build_actions(&mut self, all_data: &AllData) -> Vec<String> {

                let nearest_eggs = self.path_finder.nearest_eggs(
                    all_data.my_base_index,
                    &all_data.cellules
                );
                let nearest_crystals = self.path_finder.nearest_crystals(
                    all_data.my_base_index,
                    &all_data.cellules
                );

                eprintln!("index nearest eggs {}", nearest_eggs);
                eprintln!("index nearest crys {}", nearest_crystals);

                let collect_eggs_action = format!(
                    "LINE {} {} {}",
                    all_data.my_base_index,
                    nearest_eggs,
                    1
                );
                let collect_crystal_action = format!(
                    "LINE {} {} {}",
                    all_data.my_base_index,
                    nearest_crystals,
                    1
                );

                if all_data.tour_actuel < 7 {
                    let collect_eggs_action = format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        nearest_eggs,
                        2
                    );

                    vec![collect_eggs_action]
                } else if all_data.tour_actuel < 12 {
                    let collect_eggs_action = format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        nearest_eggs,
                        2
                    );

                    let collect_crystal_action = format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        nearest_crystals,
                        2
                    );

                    vec![
                        collect_eggs_action,
                        collect_crystal_action
                    ]
                } else {
                    let collect_crystal_action = format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        nearest_crystals,
                        4
                    );

                    vec![
                        collect_crystal_action
                    ]
                }
            }
        }

        impl BasicIAWithPathFinder {
            pub fn new() -> Self {
                Self {
                    current_target: None,
                    path_finder: Box::new(BrutalPathFinder {})
                }
            }
        }
    }
}

mod path_finders {

    pub mod brutal {
        use crate::core::path_finders::CanGiveBestTarget;
        use crate::models::Cellule;
        use crate::type_redifined::Identifiant;

        pub struct BrutalPathFinder;

        impl CanGiveBestTarget for BrutalPathFinder {
            fn nearest_eggs(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> Identifiant {
                self.nearest_element(base_index, cellules, vec![],1, 1).0
            }

            fn nearest_crystals(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> Identifiant {
                self.nearest_element(base_index, cellules, vec![], 2, 1).0
            }
        }

        impl BrutalPathFinder {

            pub fn new() -> Self {
                Self {}
            }

            fn nearest_element(
                &self,
                base_index: Identifiant,
                cellules: &Vec<Cellule>,
                deja_vu: Vec<Identifiant>,
                r#type: i32,
                iteration: i32
            ) -> (Identifiant, i32) {
                // eprintln!("iteration {}", iteration);

                let current_cellule = cellules.iter()
                    .find(|c| c.identifiant == base_index)
                    .unwrap();

                let voisin_existants = current_cellule.voisins.iter()
                    .map(|e| e.clone())
                    .filter(|index| index.clone() != -1 && !deja_vu.contains(index))
                    .collect::<Vec<_>>();

                let voisin_avec_element = voisin_existants
                    .iter()
                    .map(|e| e.clone())
                    .find(|index| {
                        cellules.iter()
                            .find(|cellule| cellule.identifiant == index.clone() && cellule.r#type == r#type && cellule.nombre_de_crystal > 0)
                            .is_some()
                    });

                if voisin_avec_element.is_some() {
                    (voisin_avec_element.unwrap(), iteration + 1)
                } else {
                    let mut res = voisin_existants.iter()
                        .map(|voisin_index| {
                            self.nearest_element(
                                voisin_index.clone(),
                                cellules,
                                deja_vu.clone().into_iter()
                                    .chain(voisin_existants.clone().into_iter())
                                    .collect::<Vec<_>>(),
                                r#type,
                                iteration + 1
                            )
                        })
                        .filter(|e| e.clone().1 != -1)
                        .collect::<Vec<_>>();

                    res.sort_by(|e1, e2| (*e1).1.cmp(&(*e2).1));
                    res
                        .iter()
                        .map(|e| e.clone())
                        .filter(|e| {
                            let cellule = cellules.iter()
                                .map(|cell| cell.clone())
                                .find(|cell| cell.identifiant == (e.clone().0));
                            cellule.unwrap().nombre_de_crystal > 0
                        })
                        .nth(0)
                        .unwrap_or((-1, -1))
                }
            }
        }

    }
}

mod core {
    pub mod behaviors {
        use crate::models::AllData;

        pub trait CanBuildActions {
            fn build_actions(&mut self, all_data: &AllData) -> Vec<String>;

            fn execute_actions(&mut self, all_data: &AllData) {
                let actions = self.build_actions(all_data).join(";");
                println!("{}", actions);
            }
        }

    }

    pub mod path_finders {
        use crate::models::Cellule;
        use crate::type_redifined::Identifiant;

        pub trait CanGiveBestTarget {
            fn nearest_eggs(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> Identifiant;
            fn nearest_crystals(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> Identifiant;
        }

    }
}

mod models {

    pub struct AllData {
        pub cellules: Vec<Cellule>,
        pub my_base_index: i32,
        pub opp_base_index: i32,
        pub tour_actuel: i32
    }

    #[derive(Clone)]
    pub struct Cellule {
        pub r#type: i32,
        pub identifiant: i32,
        pub nombre_de_crystal: i32,
        pub nombre_insectes: Option<i32>,
        pub nombre_insectes_enemy: Option<i32>,
        pub voisins: Vec<i32>
    }
}

mod helpers {


    pub trait CanSort<T> {
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
                cloned.reverse();
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
                    nombre_insectes: None,
                    nombre_insectes_enemy: None,
                    voisins: vec![
                        neigh_0,
                        neigh_1,
                        neigh_2,
                        neigh_3,
                        neigh_4,
                        neigh_5
                    ]
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn update_cellules(initial_cellules: &Vec<Cellule>) -> Vec<Cellule> {
        (0..initial_cellules.len())
            .into_iter()
            .map(|index| {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();
                let inputs = input_line.split(" ").collect::<Vec<_>>();
                let resources = parse_input!(inputs[0], i32); // the current amount of eggs/crystals on this cell
                let my_ants = parse_input!(inputs[1], i32); // the amount of your ants on this cell
                let opp_ants = parse_input!(inputs[2], i32); // the amount of opponent ants on this cell

                let initial_cellule = initial_cellules
                    .iter()
                    .find(|cellule| cellule.identifiant == index as i32)
                    .unwrap();

                Cellule {
                    r#type: initial_cellule.r#type,
                    identifiant: initial_cellule.identifiant,
                    nombre_de_crystal: resources,
                    nombre_insectes: Some(my_ants),
                    nombre_insectes_enemy: Some(opp_ants),
                    voisins: initial_cellule.voisins.clone()
                }
            })
            .collect::<Vec<_>>()
    }
}

mod type_redifined {
    pub type Identifiant = i32;
}
