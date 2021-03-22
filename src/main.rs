#![feature(hash_drain_filter)]

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt::Debug;

#[derive(PartialEq, Eq)]
enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}
#[derive(PartialEq, Eq)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Copy, Clone)]
struct Card(u8);

impl Card {
    pub fn rank(&self) -> Rank {
        match self.0 % 13 {
            0 => Rank::Ace,
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            8 => Rank::Nine,
            9 => Rank::Ten,
            10 => Rank::Jack,
            11 => Rank::Queen,
            12 => Rank::King,
            _ => panic!("Card out of range"),
        }
    }
    pub fn suit(&self) -> Suit {
        match self.0 / 13 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => panic!("Card out of range"),
        }
    }
}
impl Rank {
    pub fn single_char(self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
        }
    }
}
impl Suit {
    pub fn single_char(self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }
}
impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.rank().single_char(),
            self.suit().single_char()
        )
    }
}
#[derive(Clone)]
struct Deck {
    list: Vec<Card>,
    pos: usize,
}
impl Debug for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.list
                .iter()
                .skip(self.pos)
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
impl Deck {
    pub fn new_unshuffled() -> Self {
        Self {
            pos: 0,
            list: (0..52).map(|x| Card(x)).collect(),
        }
    }
    pub fn new_shuffled() -> Self {
        let mut d = Self::new_unshuffled();
        d.list.shuffle(&mut thread_rng());
        d
    }
    pub fn draw(&mut self) -> Option<Card> {
        if self.pos >= self.list.len() {
            None
        } else {
            self.pos += 1;
            Some(self.list[self.pos - 1])
        }
    }
}

#[derive(Clone)]
struct Game {
    deck: Deck,
    choice_points: usize,
    tableau: Vec<PlacedCard>,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MatchType {
    Suit,
    Rank,
}
type MatchDistance = u8;
#[derive(Debug)]
enum Choices {
    GameWon,
    GameLost,
    ChooseOne(Vec<Match>),
}
type Match = (usize, MatchDistance);

#[derive(Clone)]
struct PlacedCard {
    card: Card,
    matches_one: bool,
    matches_three: bool,
}
impl Debug for PlacedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match (self.matches_one, self.matches_three) {
            (false, false) => "_",
            (true, false) => "S",
            (false, true) => "L",
            (true, true) => "B",
        };
        write!(f, "{:?}{}", self.card, c)
    }
}

struct SavedGame {
    pos: usize,
    tableau: Vec<PlacedCard>,
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Deck: {:?}", self.deck)?;
        writeln!(
            f,
            "Tableau: {}",
            self.tableau
                .iter()
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
impl<'a> Game {
    pub fn new() -> Self {
        Self {
            deck: Deck::new_shuffled(),
            tableau: Vec::new(),
            choice_points: 0,
        }
    }
    pub fn save_game(&'a self) -> SavedGame {
        SavedGame {
            pos: self.deck.pos,
            tableau: self.tableau.clone(),
        }
    }
    pub fn restore(&'a mut self, saved: SavedGame) {
        self.deck.pos = saved.pos;
        self.tableau = saved.tableau;
    }
    pub fn deal_card(&mut self) -> Option<()> {
        let c = self.deck.draw()?;
        self.tableau.push(PlacedCard{card: c, matches_one: false, matches_three: false});
        self.check_matches_at(self.tableau.len() - 1);
        Some(())
    }
    pub fn remove_card(&mut self, ix: usize) -> Card {
        //removing card at ix: need to reconsider cards at indices ix, ix+1,ix+2
        let c = self.tableau.remove(ix);
        for ix in ix..=ix+2 {
            self.check_matches_at(ix);
        }
        c.card
    }

    pub fn place_card(&mut self, c: Card, ix: usize) {
        //placing a card at ix (overwrite) or on the end.
        self.tableau[ix] = PlacedCard{card: c, matches_three: false, matches_one: false};
        for ix in ix..=ix+2 {
            self.check_matches_at(ix);
        }
    }
    fn check_matches_at(&mut self, ix: usize) {
        //card at ix has just changed. check for new matches going left.
        let n = self.tableau.len();
        if ix >= n {
            return;
        }
        let a = &self.tableau[ix];
        let m1 = self
            .tableau
            .get(ix.wrapping_sub(1))
            .and_then(|b| Self::is_match(a, b))
            .is_some();
        let m3 = self
            .tableau
            .get(ix.wrapping_sub(3))
            .and_then(|b| Self::is_match(a, b))
            .is_some();
        let x = self.tableau.get_mut(ix).unwrap();
        x.matches_one = m1;
        x.matches_three = m3;
    }
    fn is_match(a: &PlacedCard, b: &PlacedCard) -> Option<MatchType> {
        if a.card.suit() == b.card.suit() {
            Some(MatchType::Suit)
        } else if a.card.rank() == b.card.rank() {
            Some(MatchType::Rank)
        } else {
            None
        }
    }
    pub fn find_matches(&mut self) -> Vec<Match> {
        let mut ans = Vec::new();
        for (ix, c) in self.tableau.iter().enumerate() {
            if c.matches_one{
                ans.push((ix, 1));
            }
            if c.matches_three {
                ans.push((ix, 3));
            }
        }
        ans
    }
    fn make_match(&mut self, m: Match) {
        let from = m.0;
        let d : usize = m.1.into();
        let to: usize = m.0 - d;
        let picked_up = self.remove_card(from);
        self.place_card(picked_up, to);
    }
    pub fn make_choice(&mut self, m: Match) {
        self.make_match(m);
    }

    pub fn play_to_choice(&mut self) -> Choices {
        loop {
            let choices = self.find_matches();
            //println!("{:?}", self);
            match choices.len() {
                0 => match self.deal_card() {
                    Some(_) => {}
                    None => {
                        if self.tableau.len() == 1 {
                            return Choices::GameWon;
                        } else {
                            return Choices::GameLost;
                        }
                    }
                },
                1 => self.make_match(choices.into_iter().next().unwrap()),
                _ => {
                    self.choice_points += 1;
                    return Choices::ChooseOne(choices);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Result {
    AlwaysWin,
    AlwaysLose,
    GaveUp,
    CanWin
}
pub fn play_one() -> (usize, Result) {
    let mut g = Game::new();
    let mut losses = 0;
    let mut wins = 0;
    let mut to_retry = Vec::new();
    loop {
        let mut choices = 0;
        match g.play_to_choice() {
            Choices::GameWon => {
                wins += 1;
            }
            Choices::GameLost => {
                losses += 1;
            }
            Choices::ChooseOne(c) => {
                for ch in c {
                    to_retry.push((g.save_game(), ch));
                }
                if g.choice_points > 1_000_000 {
                    return (g.choice_points,Result::GaveUp)
                }
            }
        }
        if let Some(x) = to_retry.pop() {
            g.restore(x.0);
            g.make_choice(x.1);
        } else {
            break;
        }
    }
    if losses == 0 {
        (g.choice_points, Result::AlwaysWin)
    } else if wins == 0 {
        (g.choice_points, Result::AlwaysLose)
    } else {
        (g.choice_points, Result::CanWin)
    }
}
fn main() {
    let mut losses = 0;
    let mut wins = 0;
    let mut maybe_wins = 0;
    let mut too_hard = 0;
    let mut games = 0;
    loop {
        games += 1;
        let (g, r) = play_one();
        match r {
            Result::AlwaysWin => {
                wins += 1;
            }
            Result::AlwaysLose => {
                losses += 1;
            }
            Result::CanWin => {
                maybe_wins += 1;
            }
            Result::GaveUp => {
                too_hard += 1;
            }
        }
        println!("Always win {}, Always lose {}, Can win {}, Gave up on {} out of {} games. Last game had {} choice points",wins,losses,maybe_wins, too_hard, games,g);
    }
}
