//! # TicTacToe
//! A small example of using the [chameleon] framework to turn a simple tic-tac-toe
//! game logic into a fully-fledged bot.

use chameleon::mcts::MonteCarloTree;

pub fn main() {
    println!("Chameleon-TicTacToe example\n");
    let mut board = TicTacToe::new();
    let mut mcts = MonteCarloTree::new();
    println!("{board}\n");

    let stdin = std::io::stdin();
    let input = &mut String::new();

    // Player choice
    println!("Do you want to play as cross (x) or circle (o)?");
    let player = loop {
        input.clear();
        stdin.read_line(input).unwrap();

        match input.to_lowercase().as_str().trim() {
            "x" => break Tick::Cross,
            "o" => break Tick::Circle,
            i => println!("{i} is not a valid answer, either x to play cross or o to play circle."),
        }
    };
    println!();

    // Main game loop
    let winner = loop {
        // Check wether any player won
        match board.player_has_won() {
            Tick::None if !board.available_squares().is_empty() => {}
            t => break t,
        }

        // If not, then either the bot or the player does something
        if player == board.currently_playing() {
            println!("Your turn, pick the square you want to mark:");
            let square: usize = loop {
                input.clear();
                stdin.read_line(input).unwrap();

                match input.trim().parse::<usize>() {
                    Ok(num) if board.available_squares().contains(&num) => break num,
                    _ => {
                        println!("Come on, {} is not a valid square:", input.trim())
                    }
                }
            };

            println!("You marked square {square}");
            board.mark(square);
        } else {
            for _ in 0..1600 {
                mcts.step(&mut board);
            }
            let action = mcts
                .best_action(&mut board)
                .unwrap_or_else(|| panic!("The bot broke :("));

            board.mark(action);
            println!("The bot marked square {action}");
        }

        println!("{board}\n");
    };

    if winner == player {
        println!("You won, yay, good job!")
    } else if winner != Tick::None {
        println!("You lost, meh, the bot was cheating anyway")
    } else {
        println!("This is a draw, decent enough")
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
pub enum Tick {
    Cross,
    Circle,
    #[default]
    None,
}
impl std::fmt::Display for Tick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cross => 'x',
                Self::Circle => 'o',
                Self::None => ' ',
            }
        )
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct TicTacToe {
    board: [Tick; 9],
    circle_to_play: bool,

    history: [usize; 9],
    moves: usize,
}
impl TicTacToe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn empty_squares(&self) -> usize {
        self.board.iter().filter(|t| **t == Tick::None).count()
    }

    pub fn mark(&mut self, square: usize) {
        if self.moves == 9 {
            return;
        }

        self.history[self.moves] = square;
        self.board[square] = if self.circle_to_play {
            Tick::Circle
        } else {
            Tick::Cross
        };

        self.circle_to_play = !self.circle_to_play;
        self.moves += 1;
    }

    pub fn unmark(&mut self) {
        self.moves -= 1;
        let square = self.history[self.moves];
        self.history[self.moves] = 0;
        self.board[square] = Tick::None;
        self.circle_to_play = !self.circle_to_play
    }

    pub fn currently_playing(&self) -> Tick {
        if self.circle_to_play {
            Tick::Circle
        } else {
            Tick::Cross
        }
    }

    pub fn available_squares(&self) -> Vec<usize> {
        self.board
            .iter()
            .enumerate()
            .filter_map(|(sq, tick)| if *tick == Tick::None { Some(sq) } else { None })
            .collect()
    }

    pub fn player_has_won(&self) -> Tick {
        // Find any line
        if self.board[0..3]
            .iter()
            .all(|t| *t == self.board[0] && *t != Tick::None)
        {
            return self.board[0];
        }
        if self.board[3..6]
            .iter()
            .all(|t| *t == self.board[3] && *t != Tick::None)
        {
            return self.board[3];
        }
        if self.board[6..9]
            .iter()
            .all(|t| *t == self.board[6] && *t != Tick::None)
        {
            return self.board[6];
        }

        // Find any column
        if self.board[0..9]
            .iter()
            .step_by(3)
            .all(|t| *t == self.board[0] && *t != Tick::None)
        {
            return self.board[0];
        }
        if self.board[1..9]
            .iter()
            .step_by(3)
            .all(|t| *t == self.board[1] && *t != Tick::None)
        {
            return self.board[1];
        }
        if self.board[2..9]
            .iter()
            .step_by(3)
            .all(|t| *t == self.board[2] && *t != Tick::None)
        {
            return self.board[2];
        }

        // Find any diagonal
        if self.board[0] == self.board[4]
            && self.board[0] == self.board[8]
            && self.board[0] != Tick::None
        {
            return self.board[0];
        }
        if self.board[2] == self.board[4]
            && self.board[2] == self.board[6]
            && self.board[2] != Tick::None
        {
            return self.board[2];
        }
        Tick::None
    }
}

impl chameleon::game::Game for TicTacToe {
    type Action = usize;
    type Hash = Self;
    type ActionsIter = Vec<usize>;
    type Player = Tick;

    fn play(&mut self, action: &Self::Action) {
        self.mark(*action)
    }
    fn undo(&mut self) {
        self.unmark()
    }
    fn actions(&self) -> Self::ActionsIter {
        self.available_squares()
    }
    fn hash(&self) -> Self::Hash {
        *self
    }
    fn current_player(&self) -> Self::Player {
        self.currently_playing()
    }
    fn utility(&self) -> chameleon::game::Utility<Self> {
        match self.player_has_won() {
            Tick::Cross => {
                chameleon::game::Utility::Exact(chameleon::game::ExactUtility::Win(Tick::Cross))
            }
            Tick::Circle => {
                chameleon::game::Utility::Exact(chameleon::game::ExactUtility::Win(Tick::Circle))
            }
            Tick::None => {
                if self.available_squares().is_empty() {
                    chameleon::game::Utility::Exact(chameleon::game::ExactUtility::Draw)
                } else {
                    chameleon::game::Utility::Unknown
                }
            }
        }
    }
}

impl std::fmt::Display for TicTacToe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, tick) in self.board[0..3].iter().enumerate() {
            write!(
                f,
                "{} ",
                if *tick == Tick::None {
                    i.to_string()
                } else {
                    tick.to_string()
                }
            )?
        }
        writeln!(f)?;
        for (i, tick) in self.board[3..6].iter().enumerate() {
            write!(
                f,
                "{} ",
                if *tick == Tick::None {
                    (i + 3).to_string()
                } else {
                    tick.to_string()
                }
            )?
        }
        writeln!(f)?;
        for (i, tick) in self.board[6..9].iter().enumerate() {
            write!(
                f,
                "{} ",
                if *tick == Tick::None {
                    (i + 6).to_string()
                } else {
                    tick.to_string()
                }
            )?
        }
        Ok(())
    }
}
