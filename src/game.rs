use std::hash::Hash;

/// Utility of nodes can be either:
/// - [Exact], meaning the value is known precisely. This score is given to terminal nodes or nodes with only children with known utilities.
/// - [Approximate] is a value giving an idea of what this node's value could be. It can either be given by an approximation function or by random playouts.
/// - [Unknown] is a value indicating that random playouts must be used to determine the nodes [Appoximate] value.
///
/// Note that approximation scores are quantized, that is to say they represent a signed percentage.
/// a value of `i16::MAX` means that the side to move has a 100% chance of winning,
/// while a value of `i16::MIN` means that the side to move has a 100% chance of loosing.
#[derive(PartialEq, Hash, Eq)]
pub enum Utility<G: Game> {
    Exact(ExactUtility<G>),
    Approximate(i16),
    Unknown,
}
impl<G: Game> Clone for Utility<G> {
    fn clone(&self) -> Self {
        match self {
            Self::Exact(e) => Self::Exact(*e),
            Self::Approximate(i) => Self::Approximate(*i),
            Self::Unknown => Self::Unknown,
        }
    }
}
impl<G: Game> Copy for Utility<G> {}

#[derive(PartialEq, Hash, Eq)]
pub enum ExactUtility<G: Game> {
    Win(G::Player),
    Draw,
}
impl<G: Game> Clone for ExactUtility<G> {
    fn clone(&self) -> Self {
        match self {
            Self::Win(p) => Self::Win(*p),
            Self::Draw => Self::Draw,
        }
    }
}
impl<G: Game> Copy for ExactUtility<G> {}

/// The [Game] trait is meant to describe a (potentially infinite) game tree in
/// a way that is usable by the MCTS algorithm.
pub trait Game: Sized {
    type Action: PartialEq + Eq;
    type ActionsIter: IntoIterator<Item = Self::Action>;
    type Hash: Hash + Eq + PartialEq;
    type Player: Clone + Copy + PartialEq + Hash + Eq;

    fn play(&mut self, action: &Self::Action);
    fn undo(&mut self);

    fn current_player(&self) -> Self::Player;
    fn actions(&self) -> Self::ActionsIter;

    fn utility(&self) -> Utility<Self>;
    fn hash(&self) -> Self::Hash;
}
