use std::cell::Cell;
use std::io;
use crate::helpers::*;

use crate::behaviors::basic_ia_with_bronze::BasicIABronze;
use crate::core::behaviors::CanBuildActions;
use crate::core::path_finders::CanFindDistanceIndex;

use crate::models::{AllData, Cellule};

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn main() {

    let initial_cellules = load_cellules();
    let _ = load_nombre_de_bases();
    let my_base_index = load_index_base();
    let opp_base_index = load_index_base();

    // ajout des distances dans les cellules
    struct CalculateurDistance;
    impl CanFindDistanceIndex for CalculateurDistance {};
    let calulateur_distance = CalculateurDistance {};
    let initial_cellules_with_distance = initial_cellules
        .iter()
        // .filter(|cellule| cellule.r#type != 0)
        .map(|cellule| {
            // on calcule que la distance où le type est pas vide a cause du temps de calcul
            let distance_base_opt = if cellule.r#type != 0 {
                let distance_base = calulateur_distance.find_distance(
                    my_base_index,
                    &initial_cellules,
                    cellule.identifiant
                );
                Some(distance_base)
            } else {
                None
            };
            Cellule {
                r#type: cellule.r#type,
                identifiant: cellule.identifiant,
                nombre_de_crystal: cellule.nombre_de_crystal,
                nombre_insectes: cellule.nombre_insectes,
                nombre_insectes_enemy: cellule.nombre_insectes_enemy,
                voisins: cellule.voisins.clone(),
                distance_base: distance_base_opt
            }
        })
        .collect::<Vec<_>>();

    let my_bot = BasicIABronze :: new();

    let mut index_tour = 0;

    // game loop
    loop {
        index_tour += 1;

        // WAIT | LINE <sourceIdx> <targetIdx> <strength> | BEACON <cellIdx> <strength> | MESSAGE <text>
        let updated_cellules = update_cellules(&initial_cellules_with_distance);

        let all_data = AllData {
            initial_cellules: initial_cellules_with_distance.clone(),
            cellules: updated_cellules,
            my_base_index,
            opp_base_index,
            tour_actuel: index_tour
        };

        my_bot.execute_actions(&all_data);
    }
}

mod behaviors {

    pub mod basic_ia_attrape_tout_crystaux {
        use crate::core::behaviors::CanBuildActions;
        use crate::helpers::CanSort;
        use crate::models::{AllData, Cellule};

        pub struct BasicIAAttrapeToutCrystaux;

        impl CanBuildActions for BasicIAAttrapeToutCrystaux {
            fn build_actions(&self, all_data: &AllData) -> Vec<String> {
                let sorted_cellules_by_crystal = all_data
                    .cellules.clone()
                    .into_iter()
                    .filter(|cellule| cellule.nombre_de_crystal > 0 && cellule.r#type == 2)
                    .collect::<Vec<_>>()
                    .sort_immut();

                sorted_cellules_by_crystal
                    .into_iter()
                    .map(|cellule| {
                        format!(
                            "LINE {} {} {}",
                            all_data.my_base_index,
                            cellule.identifiant,
                            20
                        )
                    })
                    .collect::<Vec<_>>()
            }
        }

        impl BasicIAAttrapeToutCrystaux {
            pub fn new() -> Self {
                Self {}
            }
        }
    }

    pub mod basic_ia_recherche_nid_proche {
        use crate::core::behaviors::CanBuildActions;
        use crate::core::path_finders::{CanFindDistanceIndex, CanGiveBestTarget};
        use crate::models::AllData;

        pub struct BasicIARechercheNidProche;

        impl CanFindDistanceIndex for BasicIARechercheNidProche {}

        impl CanGiveBestTarget for BasicIARechercheNidProche {}

        impl CanBuildActions for BasicIARechercheNidProche {
            fn build_actions(&self, all_data: &AllData) -> Vec<String> {
                let nearest_eggs = self.nearest_eggs(
                    all_data.my_base_index,
                    &all_data.cellules
                );
                let nearest_egg_id = nearest_eggs.0;

                eprintln!("index nearest eggs {:?}", nearest_eggs);

                let action = if nearest_egg_id > -1 {
                    format!(
                        "LINE {} {} {}",
                        all_data.my_base_index,
                        nearest_egg_id,
                        10
                    )
                } else {
                    "WAIT".to_string()
                };

                vec![
                    action
                ]
            }
        }

        impl BasicIARechercheNidProche {
            pub fn new() -> Self {
                Self {}
            }
        }
    }

    pub mod basic_ia_with_bronze {
        use crate::behaviors::basic_ia_attrape_tout_crystaux::BasicIAAttrapeToutCrystaux;
        use crate::behaviors::basic_ia_recherche_nid_proche::BasicIARechercheNidProche;
        use crate::core::behaviors::CanBuildActions;
        use crate::models::AllData;

        pub struct BasicIABronze {
            ia_attrape_tout: Box<dyn CanBuildActions>,
            ia_first_nid: Box<dyn CanBuildActions>
        }

        impl CanBuildActions for BasicIABronze {
            fn build_actions(&self, all_data: &AllData) -> Vec<String> {
                let nombre_total_insect = all_data.get_my_total_insect();

                if all_data.get_nombre_nid_detruit() < 1 && nombre_total_insect < 30 {
                    self.ia_first_nid.build_actions(all_data)
                } else {
                    self.ia_attrape_tout.build_actions(all_data)
                }
            }
        }

        impl BasicIABronze {
            pub fn new() -> Self {
                Self {
                    ia_attrape_tout: Box::new(BasicIAAttrapeToutCrystaux::new()),
                    ia_first_nid: Box::new(BasicIARechercheNidProche::new())
                }
            }
        }
    }
}

mod core {
    pub mod behaviors {
        use crate::models::{AllData, Cellule};
        use crate::type_redifined::Identifiant;

        pub trait CanBuildActions {
            fn build_actions(&self, all_data: &AllData) -> Vec<String>;

            fn execute_actions(&self, all_data: &AllData) {
                let actions = self.build_actions(all_data).join(";");
                println!("{}", actions);
            }
        }

    }

    pub mod path_finders {
        use crate::core::path_finders::distance_helper::calcul_distance;
        use crate::models::Cellule;
        use crate::type_redifined::{Distance, Identifiant};

        pub trait CanGiveBestTarget: CanFindDistanceIndex {

            fn nearest_resources(
                &self,
                base_index: Identifiant,
                cellules: &Vec<Cellule>,
                deja_vu: Vec<Identifiant>,
                r#type: i32,
                iteration: i32
            ) -> (Identifiant, Distance) {

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
                            self.nearest_resources(
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
            fn nearest_eggs(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> (Identifiant, Distance) {
                self.nearest_resources(base_index, cellules, vec![],1, 1)
            }
            fn nearest_crystals(&self, base_index: Identifiant, cellules: &Vec<Cellule>) -> (Identifiant, Distance) {
                self.nearest_resources(base_index, cellules, vec![],2, 1)
            }
        }

        pub trait CanFindDistanceIndex {
            fn find_distance(&self, depart_identifiant: Identifiant, cellules: &Vec<Cellule>, recherche_id: Identifiant) -> Distance {
                calcul_distance(
                    depart_identifiant,
                    cellules,
                    recherche_id,
                    vec![],
                    0
                )
            }
        }

        mod distance_helper {
            use crate::models::Cellule;
            use crate::type_redifined::{Distance, Identifiant};

            pub fn calcul_distance(
                depart_index: Identifiant,
                cellules: &Vec<Cellule>,
                recherche_id: Identifiant,
                deja_vu: Vec<Identifiant>,
                iteration: Distance
            ) -> Distance {
                let current_cellule = cellules.iter()
                    .find(|c| c.identifiant == depart_index)
                    .unwrap();

                let voisin_existants = current_cellule.voisins.iter()
                    .map(|e| e.clone())
                    .filter(|index| index.clone() != -1 && !deja_vu.contains(index))
                    .collect::<Vec<_>>();

                if voisin_existants.contains(&recherche_id) {
                    iteration + 1
                } else {
                    let mut res = voisin_existants.iter()
                        .map(|voisin_index| {
                            calcul_distance(
                                voisin_index.clone(),
                                cellules,
                                recherche_id,
                                deja_vu.clone().into_iter()
                                    .chain(voisin_existants.clone().into_iter())
                                    .collect::<Vec<_>>(),
                                iteration + 1
                            )
                        })
                        .filter(|e| e.clone() != -1)
                        .collect::<Vec<_>>();

                    res.sort_by(|e1, e2| e1.cmp(e2));
                    res
                        .iter()
                        .map(|e| e.clone())
                        .nth(0)
                        .unwrap_or(-1)
                }
            }
        }
    }
}

mod models {
    use crate::type_redifined::{Distance, Identifiant};

    pub struct AllData {
        pub initial_cellules: Vec<Cellule>,
        pub cellules: Vec<Cellule>,
        pub my_base_index: i32,
        pub opp_base_index: i32,
        pub tour_actuel: i32
    }

    impl AllData {
        pub fn get_my_total_insect(&self) -> i32 {
            self.cellules.iter().fold(0, |acc, current| {
                acc + current.nombre_insectes.unwrap_or(0)
            })
        }

        pub fn get_enemy_total_insect(&self) -> i32 {
            self.cellules.iter().fold(0, |acc, current| {
                acc + current.nombre_insectes_enemy.unwrap_or(0)
            })
        }

        pub fn get_nombre_nid_detruit(&self) -> i32 {
            self.get_nombre_nid_initial() - self.get_nombre_nid_actuel()
        }

        pub fn get_nombre_crystal_detruit(&self) -> i32 {
            self.get_nombre_crystal_initial() - self.get_nombre_crystal_actuel()
        }

        pub fn get_nombre_crystal_initial(&self) -> i32 {
            self.initial_cellules
                .iter()
                .filter(|cellule| cellule.r#type == 2)
                .count() as i32
        }

        pub fn get_nombre_crystal_actuel(&self) -> i32 {
            self.cellules
                .iter()
                .filter(|cellule| cellule.r#type == 2 && cellule.nombre_de_crystal > 0)
                .count() as i32
        }

        pub fn get_nombre_nid_initial(&self) -> i32 {
            self.initial_cellules
                .iter()
                .filter(|cellule| cellule.r#type == 1)
                .count() as i32
        }

        pub fn get_nombre_nid_actuel(&self) -> i32 {
            self.cellules
                .iter()
                .filter(|cellule| cellule.r#type == 1 && cellule.nombre_de_crystal > 0)
                .count() as i32
        }
    }

    #[derive(Clone)]
    pub struct Cellule {
        pub r#type: i32,
        pub identifiant: Identifiant,
        pub nombre_de_crystal: i32,
        pub nombre_insectes: Option<i32>,
        pub nombre_insectes_enemy: Option<i32>,
        pub voisins: Vec<Identifiant>,
        pub distance_base: Option<Distance>
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
    use crate::core::path_finders::CanFindDistanceIndex;
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
                    ],
                    distance_base: None
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
                    voisins: initial_cellule.voisins.clone(),
                    distance_base: initial_cellule.distance_base
                }
            })
            .collect::<Vec<_>>()
    }
}

mod type_redifined {
    pub type Identifiant = i32;
    pub type Distance = i32;
}
