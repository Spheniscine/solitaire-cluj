use std::time::Duration;

use rand::{Rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::game::{Board, BoardPos, Card, DECK_SIZE, DepotRole, NUM_RANKS, NUM_SUITS, RANKS, Skin, Suit};

pub const ANIMATION_DURATION: Duration = Duration::from_millis(200);
pub type AnimationKey = u16;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ActionRecord {
    pos1: BoardPos, pos2: BoardPos, tap: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ScreenState {
    #[default] Game, 
    Settings, Help,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameState {
    pub board: Board,
    pub deal: Vec<Card>,
    #[serde(skip)]
    pub animation_key: AnimationKey, // used for syncing and to provide animator components with cycling keys
    pub history: Vec<ActionRecord>,
    pub undo_stack: Vec<usize>,
    pub already_won: bool,
    pub num_wins: i32,

    pub screen_state: ScreenState,

    pub allow_undo: bool,
    pub skin: Skin,
}

impl GameState {
    pub fn new_deal(rng: &mut impl Rng) -> Vec<Card> {
        let mut deck = Vec::with_capacity(DECK_SIZE);
        for rank in RANKS {
            for suit in Suit::iter() {
                deck.push(Card { rank, suit, tapped: false });
            }
        }

        deck.shuffle(rng);
        deck
    }

    pub fn new_game(&mut self) {
        let deal = Self::new_deal(&mut rand::rng());
        self.board = Board::from_deal(&deal);
        self.deal = deal;
        self.history.clear();
        self.undo_stack.clear();
        self.already_won = false;
        // LocalStorage.save_game_state(&self);
    }

    pub fn init() -> Self {
        let mut res = Self {
            board: Board::empty(),
            deal: vec![],
            animation_key: 0,
            history: vec![],
            undo_stack: vec![],
            already_won: false,
            num_wins: 0,
            screen_state: ScreenState::Game,
            allow_undo: true,
            skin: Skin::default(),
        };

        res.new_game();

        // // test for column limit
        // for i in (1..=8).rev() {
        //     let card = Card { rank: i, suit: Suit::Clubs, tapped: i == 1 };
        //     res.board.depots[DepotRole::Tableau.id(0)].push(card);
        // }

        res
    }

    pub fn is_busy(&self) -> bool {
        self.is_acting()
    }

    pub fn is_acting(&self) -> bool {
        !self.board.animation_acts.is_empty()
    }

    pub fn undo_possible(&self) -> bool {
        self.allow_undo && !self.undo_stack.is_empty()
    }

    pub fn is_won(&self) -> bool {
        DepotRole::Shadow.range().filter(|&s| {
            self.board.depots[s].len() == NUM_RANKS
        }).count() == NUM_SUITS
    }

    pub fn can_stack(&self, back: Card, front: Card) -> bool {
        !back.tapped && front.rank + 1 == back.rank
    }

    pub fn can_cheat(&self, back: Card, front: Card) -> bool {
        !back.tapped && !front.tapped
    }

    fn is_stack(&self, slice: &[Card]) -> bool {
        slice.windows(2).all(|w| self.can_stack(w[0], w[1]))
    }

    pub fn can_select(&self, pos: BoardPos) -> bool {
        let depot = pos.depot_index;
        let ord = pos.card_index;

        if ord >= self.board.depots[depot].len() {
            return false;
        }
        let slice = &self.board.depots[depot][ord..];

        let Some(role) = DepotRole::role(depot) else { return false };
        match role {
            DepotRole::Tableau => {
                self.is_stack(slice)
            },
            DepotRole::Shadow => false,
        }
    }

    pub fn onclick(&mut self, pos: BoardPos) {
        if self.is_busy() { return; }

        if let Some(src) = self.board.selected {
            if pos == src { 
                self.board.selected = None; 
                return;
            }
            if src.depot_index == pos.depot_index && self.can_select(pos) {
                self.board.selected = Some(pos);
                return;
            }

            let dest = BoardPos::new(pos.depot_index, pos.card_index.wrapping_add(1));
            self.move_intent(src, dest);
        } else {
            if self.can_select(pos) {
                self.board.selected = Some(pos);
            }
        }
    }

    fn do_move_raw(&mut self, pos1: BoardPos, pos2: BoardPos, tap: Option<bool>) {
        self.board.do_move(pos1, pos2, tap);
        self.history.push(ActionRecord { pos1, pos2, tap })
    }

    fn move_intent(&mut self, pos1: BoardPos, pos2: BoardPos) -> bool {
        if pos1.depot_index == pos2.depot_index { return false; }
        let depot1 = &self.board.depots[pos1.depot_index];
        let depot2 = &self.board.depots[pos2.depot_index];
        let num_moved = depot1.len() - pos1.card_index;
        if pos2.card_index != depot2.len() { return false; }

        let card = depot1[pos1.card_index];
        let Some((role, ix)) = DepotRole::role_and_subindex(pos2.depot_index) else { return false };
        if role == DepotRole::Shadow { return false; }
        let shadow2 = DepotRole::Shadow.id(ix);
        if !self.board.depots[shadow2].is_empty() { return false; }

        let history_len = self.history.len();

        let can_stack = depot2.last().is_none_or(|&c| self.can_stack(c, card));
        if can_stack {
            let tap = if card.tapped {Some(false)} else {None};
            self.do_move_raw(pos1, pos2, tap);
        } else {
            if num_moved > 1 { return false; }
            let can_cheat = depot2.last().is_some_and(|&c| self.can_cheat(c, card));
            if !can_cheat { return false; }
            self.do_move_raw(pos1, pos2, Some(true));
            self.board.cheats_delta += 1;
        }

        self.undo_stack.push(history_len);
        true
    }

    pub fn check_auto_moves(&mut self) {
        if self.is_busy() { return; }
        
        // check for full stacks and move them to shadow zones simultaneously
        for i in 0..DepotRole::Tableau.number_of() {
            let cards = &self.board.depots[DepotRole::Tableau.id(i)];
            if cards.len() == NUM_RANKS && self.is_stack(&cards) {
                let src = BoardPos::new(DepotRole::Tableau.id(i), 0);
                let dest = self.board.top_pos(DepotRole::Shadow.id(i));
                self.do_move_raw(src, dest, None);
            }
        }
    }

    pub fn advance_animations(&mut self, key: AnimationKey) {
        if key != self.animation_key { return; }
        self.animation_key = self.animation_key.wrapping_add(1);
        
        self.board.advance_actions();

        if self.is_won() {
            if !self.already_won {
                self.num_wins += 1;
                self.already_won = true;
            }
        } else {
            self.check_auto_moves();
        }

        // if !self.is_busy() { LocalStorage.save_game_state(&self); }
    }
}