extern crate rand;

use self::rand::{Rng};
use network::{Network};
use std::hash::Hash;

use random_util::{choose_k_nums};
use std::collections::VecDeque;


/// Information of some node in the network about 
/// a local tower (Closest of a certain color).
#[derive(Clone)]
struct LocalTowerInfo {
    gateway: usize,
    distance: u64,
    tower_node: usize,
}

/// Choose nodes to be towers. We pick num_towers towers of every color. There are num_colors
/// different tower colors.
fn choose_towers<Node: Hash + Eq + Clone, R: Rng>(net: &Network<Node>, 
                  num_towers: usize, num_colors: usize, rng: &mut R) -> Vec<Vec<usize>> {

    let mut chosen_towers: Vec<Vec<usize>> = Vec::new();

    for _ in 0 .. num_colors {
        // Pick random towers for a certain color:
        let mut ctowers = choose_k_nums(num_towers, net.igraph.node_count(), rng)
            .into_iter()
            .collect::<Vec<usize>>();
        // Sort for determinism:
        ctowers.sort();
        chosen_towers.push(ctowers);
    }
    chosen_towers
}

/// Update operation: A given node is told about a path to a local tower.
struct UpdateOper {
    node: usize,
    tower_color: usize,
    tower_index: usize,
    local_tower_info: LocalTowerInfo,
}

fn init_towers_info(num_nodes: usize, num_colors: usize, num_towers: usize) ->
    Vec<Vec<Vec<Option<LocalTowerInfo>>>> {
    let mut towers_info: Vec<Vec<Vec<Option<LocalTowerInfo>>>> = Vec::new();
    for i in 0 .. num_nodes {
        towers_info.push(Vec::new());
        for color in 0 .. num_colors {
            towers_info[i].push(Vec::new());
            for tower_index in 0 .. num_towers {
                towers_info[i][color].push(None);
            }
        }
    }
    towers_info
}

/// Converge information about local towers. 
/// Every node will learn about the closest local towers
/// of every color.
fn calc_towers_info<Node: Hash + Eq + Clone>(net: &Network<Node>, 
    chosen_towers: Vec<Vec<usize>>) -> Vec<Vec<Vec<Option<LocalTowerInfo>>>> {

    let mut towers_info = init_towers_info(net.igraph.node_count(), 
                                           chosen_towers.len(),
                                           chosen_towers[0].len());

    let mut pending_opers: VecDeque<UpdateOper> = VecDeque::new();

    // Add initial update operations from all chosen towers.
    // Later the information about those towers will propagage all over the network.
    for tower_color in 0 .. chosen_towers.len() {
        for tower_index in 0 .. chosen_towers[tower_color].len() {
            let tower_node = chosen_towers[tower_color][tower_index];
            pending_opers.push_back(UpdateOper {
                node: tower_node,
                tower_color,
                tower_index,
                local_tower_info: LocalTowerInfo {
                    gateway: tower_node,
                    distance: 0,
                    tower_node,
                }
            });
        }
    }

    // Start handling pending operations:
    while let Some(oper) = pending_opers.pop_front() {
        let ltower_info_opt: &mut Option<LocalTowerInfo> = 
            &mut towers_info[oper.node][oper.tower_color][oper.tower_index];

        let should_update = match *ltower_info_opt {
            None => true, 
            Some(ref ltower_info) => {
                // Check if the new offered tower info (oper.local_tower_info) is better than the current
                // one (ltower_info):
                if (ltower_info.distance, 
                    ltower_info.gateway, 
                    ltower_info.tower_node) >
                    (oper.local_tower_info.distance, 
                     oper.local_tower_info.gateway, 
                     oper.local_tower_info.tower_node) {
                    true
                } else {
                    false
                }
            }
        };

        if !should_update {
            continue
        }

        // Update local tower information:
        *ltower_info_opt = Some(oper.local_tower_info.clone());
        // Notify all neighbors about new information:
        for nei in net.igraph.neighbors(oper.node) {
            pending_opers.push_back(UpdateOper {
                node: nei,
                tower_color: oper.tower_color,
                tower_index: oper.tower_index,
                local_tower_info: LocalTowerInfo {
                    gateway: oper.node,
                    distance: oper.local_tower_info.distance + 1,
                    tower_node: oper.local_tower_info.tower_node,
                }
            });
        }
    }

    towers_info
}
