//! # MCTS algorithm
//! Monte-Carlo Tree Search (MCTS) differs from traditional alpha-beta search in
//! that it does a **best-first search**. What this means is that it will try to
//! always focus on searching nodes that might be the best ones, and as a result does not
//! search the tree in a depth-first manner.
//!
//! To do so, it looks at searching like it would a [multi-armed bandit problem](https://en.wikipedia.org/wiki/Multi-armed_bandit).
//!
//! Since heuristic and results of past searches are needed in order to know how
//! to traverse the tree, we need to keep said search tree entirely in memory.

use rand::seq::IteratorRandom;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::game::{ExactUtility, Game, Utility};

/// A Monte-Carlo searched tree parametrized by the game it is playing.
pub struct MonteCarloTree<G: Game> {
    nodes: HashMap<G::Hash, Arc<Mutex<MonteCarloNode<G>>>>,

    // The number of random playouts when expanding a node with unknown utility.
    simulations_per_node: u32,
}
impl<G: Game> MonteCarloTree<G> {
    /// Constructs an empty search tree.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),

            simulations_per_node: 255,
        }
    }

    pub fn best_action(&self, state: &mut G) -> Option<G::Action> {
        let current_player = state.current_player();

        let mut best_action = None;
        let mut best_exploitation = None;
        for action in state.actions() {
            // Play the action
            state.play(&action);

            // If the child is expanded already, check its potential
            if let Some(child) = self.nodes.get(&state.hash()) {
                let child = child.lock().unwrap();
                match child.utility {
                    // If the node has an approximate value, compare it to the previously set
                    // best potential value.
                    Utility::Approximate(exploitation) => {
                        if best_exploitation < Some(exploitation) {
                            best_exploitation = Some(exploitation);
                            best_action = Some(action)
                        }
                    }
                    // If the node has an exact value and is a win for the current player, always choose it.
                    Utility::Exact(ExactUtility::Win(p)) if p == current_player => {
                        state.undo();
                        return Some(action);
                    }
                    // Otherwise, if an action is a win for the other player, try to avoid it at all cost.
                    Utility::Exact(ExactUtility::Win(_)) => {
                        if best_action.is_none() {
                            best_action = Some(action)
                        }
                    }
                    // In case we find a draw, there are two possibilities:
                    // - either no score is set (i.e. the only other options that were considered were losses)
                    //   in which case this is the best option yet.
                    // - a score is set (i.e. a node with an approximate value was considered)
                    //   in this case, we only choose this as our best option if the approximation
                    //   hold a really low exploration/exploitation ratio.
                    Utility::Exact(ExactUtility::Draw) => {
                        if best_exploitation.is_none() {
                            best_action = Some(action)
                        }
                    }
                    // If the node hasn't been expanded, we still take it over a certain loss
                    Utility::Unknown => {
                        if best_exploitation.is_none() {
                            best_action = Some(action)
                        }
                    }
                }
            } else if best_exploitation.is_none() {
                best_action = Some(action)
            }

            state.undo()
        }

        best_action
    }

    /// Expands the tree by proceeding to a selection/expansion/simulation/backpropagation
    /// routine.
    pub fn step(&mut self, state: &mut G) {
        // Keeps track of visited nodes for backpropagation.
        let mut visited = vec![];

        // Selection phase
        // This phase traverses the tree, searching for any unexpanded node.
        // At the end of this loop, `state` is a game state which hasn't been expanded yet.
        'selection: while let Some(node) = self.nodes.get(&state.hash()) {
            let current_player = state.current_player();
            visited.push(node.clone());
            let parent_visits = node.lock().unwrap().visits;

            // Search for the best action to make if any.
            let mut best_action = None;
            let mut best_potential_value: Option<f32> = None;
            let mut best_exact = None;
            for action in state.actions() {
                // Play the action
                state.play(&action);

                // If the child is expanded already, check its potential
                if let Some(child) = self.nodes.get(&state.hash()) {
                    let child = child.lock().unwrap();
                    match child.utility {
                        // Compute the exploration/exploitation factor
                        Utility::Approximate(exploitation) => {
                            // Exploration is given by the UCT formula.
                            let exploration =
                                2f32.sqrt() * ((parent_visits as f32).ln() / (child.visits as f32));
                            // Exploitation is given for side to move of the child node (aka opponent),
                            // so we reverse it here.
                            let potential_value =
                                exploration - ((exploitation as f32) / (u16::MAX as f32));

                            if best_potential_value < Some(potential_value) {
                                best_potential_value = Some(potential_value);
                                best_action = Some(action)
                            }
                        }
                        // If a child has an exact value, we do not need to search it further.
                        // However, we still need to check if we can win in any way, as if all
                        // of the children are assigned exact values, we can propagate it
                        // to this node.
                        Utility::Exact(exact_utility) => {
                            if best_exact
                                .map(|best| match (best, exact_utility) {
                                    // We always want to favor winning
                                    (_, ExactUtility::Win(p)) if p == current_player => true,
                                    // If we have the choice between a win for the other player
                                    // and a draw, favor the draw
                                    (ExactUtility::Win(p), ExactUtility::Draw)
                                        if p != current_player =>
                                    {
                                        true
                                    }
                                    // Otherwise, consider that the current best is better
                                    (_, _) => false,
                                })
                                .unwrap_or(true)
                            {
                                best_exact = Some(exact_utility)
                            }
                        }
                        // If a child has not been expanded yet, we always expand it
                        Utility::Unknown => {
                            break 'selection;
                        }
                    }
                } else {
                    break 'selection;
                }

                state.undo();
            }

            // If the node has a best action, play it then so that we're in an unexpanded
            // state.
            if let Some(best_action) = best_action {
                state.play(&best_action);
            }
            // Otherwise, all of its children are [Exact] nodes. In this case,
            // we can propagate this result to the parent node and choose another
            // path as this node is completely explored.
            else if let Some(best_exact) = best_exact {
                node.lock().unwrap().utility = Utility::Exact(best_exact);
                visited.pop();
                // We visited the entire tree and have found an exact value
                if visited.is_empty() {
                    return;
                }
                state.undo();
                visited.pop();
            } else {
                unreachable!("Visited a node with no successors")
            }
        }

        // Expansion phase
        // The current state is unexplored, we expand it and assign it a utility value.
        let utility = match state.utility() {
            // If the utility of this node is not known, we make random playouts to
            // assign it an approximate value.
            Utility::Unknown => self.simulate(state, self.simulations_per_node),
            u => u,
        };
        self.nodes.insert(
            state.hash(),
            Arc::new(Mutex::new(MonteCarloNode { utility, visits: 1 })),
        );

        // Backpropagation phase
        // We now transmit the change to the nodes we traversed.
        while let Some(node) = visited.pop() {
            state.undo();

            let mut node = node.lock().unwrap();
            node.visits += 1;

            match &mut node.utility {
                Utility::Approximate(approx) => {
                    let new = match utility {
                        Utility::Exact(ExactUtility::Win(p)) => {
                            if p == state.current_player() {
                                1f32
                            } else {
                                -1f32
                            }
                        }
                        Utility::Exact(ExactUtility::Draw) => 0f32,
                        Utility::Approximate(new) => new as f32 / i16::MAX as f32,
                        _ => unreachable!("the returned utility should never be unknown"),
                    };

                    *approx = ((((*approx as f32 / i16::MAX as f32) + new) / 2f32)
                        * (i16::MAX as f32)) as i16;
                }
                _ => {}
            }
        }
    }

    /// Simulates a number of games
    fn simulate(&self, state: &mut G, playouts: u32) -> Utility<G> {
        let mut rng = rand::thread_rng();
        let mut approximate_result = 0f32;
        let node_player = state.current_player();
        for _ in 0..playouts {
            // Traverse the game tree randomly until we find a terminal or approximate node.
            let mut plys = 0;
            let result = 'simulation: loop {
                // Pick random action
                let action = state.actions().into_iter().choose(&mut rng).unwrap();

                // Play it
                state.play(&action);
                plys += 1;

                match state.utility() {
                    Utility::Exact(ExactUtility::Win(player)) => {
                        break 'simulation if player == node_player { 1f32 } else { -1f32 }
                    }
                    Utility::Exact(ExactUtility::Draw) => break 'simulation 0f32,
                    Utility::Approximate(approx) => {
                        break 'simulation (approx as f32) / (i16::MAX as f32)
                    }
                    Utility::Unknown => {}
                }
            };

            // Return to the initial state.
            for _ in 0..plys {
                state.undo()
            }

            // Then change the approximate value
            approximate_result += result;
        }

        // We now compute the approximate value aka the approximate value
        // divided by the number of simulations.
        Utility::Approximate(
            (approximate_result / (self.simulations_per_node as f32) * (i16::MAX as f32)) as i16,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MonteCarloNode<G: Game> {
    utility: Utility<G>,
    visits: u32,
}
