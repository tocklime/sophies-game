use std::fmt::Debug;
use rand::thread_rng;
use rand::seq::SliceRandom;

#[derive(PartialEq,Eq)]
enum Rank { Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King}
#[derive(PartialEq,Eq)]
enum Suit { Clubs, Diamonds, Hearts, Spades}


#[derive(Copy,Clone)]
struct Card(u8);

impl Card {
    pub fn rank(&self) -> Rank {
        match self.0%13 {
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
            _ => panic!("Card out of range")
        }
    }
    pub fn suit(&self) -> Suit {
        match self.0/13 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => panic!("Card out of range")
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
impl Suit{
    pub fn single_char(self) -> &'static str {
        match self{
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠"
        }
    }
}
impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}{}",self.rank().single_char(),self.suit().single_char())
    }
}
#[derive(Clone,Debug)]
struct Deck {
    list : Vec<Card>,
    pos : usize
}
impl Deck {
    pub fn new_unshuffled()-> Self {
        Self {
            pos: 0,
            list : (0..51).map(|x| Card(x)).collect()
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
        }else {
            self.pos +=1;
            Some(self.list[self.pos - 1])
        }
    }
}


#[derive(Debug,Clone)]
struct Game {
    deck: Deck,
    tableau: Vec<Card>,
    choices: Vec<Match>
}
#[derive(Debug,Copy,Clone)]
pub enum MatchType {
    Suit,
    Rank,
}
#[derive(Debug,Copy,Clone)]
pub enum MatchDistance {
    One,
    Three,
}
#[derive(Debug)]
enum Choices {
    GameWon,
    GameLost,
    ChooseOne(Vec<Match>)
}
type Match = (MatchType, MatchDistance);

impl Game {
    pub fn new() -> Self {
        Self {
            deck: Deck::new_shuffled(),
            tableau: Vec::new(),
            choices: Vec::new()
        }
    }
    pub fn deal_card(&mut self) -> Option<()> {
        let c = self.deck.draw()?;
        self.tableau.push(c);
        Some(())
    }
    fn is_match(a: Card, b: Card) -> Option<MatchType> {
        if a.suit() == b.suit() {
            Some(MatchType::Suit)
        } else if a.rank() == b.rank() {
            Some(MatchType::Rank)
        } else {
            None
        }
    }
    pub fn find_matches(&mut self) -> Vec<Match> {
        let mut ans = Vec::new();
        let n = self.tableau.len();
        if n > 1 {
            if let Some(m) = Self::is_match(self.tableau[n - 1], self.tableau[n - 2]) {
                ans.push((m, MatchDistance::One));
            }
        }
        if n > 3 {
            if let Some(m) = Self::is_match(self.tableau[n-1], self.tableau[n-4]) {
                ans.push((m, MatchDistance::Three));
            }
        }
        ans
    }
    fn make_match(&mut self, m:Match) {
        let n = self.tableau.len();
        match m.1 {
            MatchDistance::One => {
                self.tableau[n-2] = self.tableau.pop().unwrap();
            }
            MatchDistance::Three => {
                self.tableau[n-4] = self.tableau.pop().unwrap();
            }
        }
    }
    pub fn make_choice(&mut self, m: Match) {
        self.choices.push(m);
        self.make_match(m);
    }

    pub fn play_to_choice(&mut self) -> Choices {
        loop {
            let choices = self.find_matches();
            match choices.len() {
                0 => {match self.deal_card(){
                    Some(_) => {}
                    None => {
                        if self.tableau.len() == 1 {
                            return Choices::GameWon;
                        }else {
                            return Choices::GameLost;
                        }
                    }
                }},
                1 => {
                    self.make_match(choices.into_iter().next().unwrap())
                }
                _ => {
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
    CanWinWith(Vec<Vec<Match>>)
}
pub fn play_one() -> Result {
    let mut g = Game::new();
    let mut losses = 0;
    let mut win_choices = Vec::new();
    let mut to_retry = Vec::new();
    loop {
        match g.play_to_choice() {
            Choices::GameWon => {
                win_choices.push(g.choices.clone());
            }
            Choices::GameLost => {
                losses += 1;
            }
            Choices::ChooseOne(c) => {
                for ch in c {
                    to_retry.push((g.clone(),ch));
                }
            }
        }
        if let Some(x) = to_retry.pop() {
            g = x.0;
            g.make_choice(x.1);
        } else {
            break;
        }
    }
    if losses == 0 {
        Result::AlwaysWin
    } else if win_choices.is_empty() {
        Result::AlwaysLose
    } else {
        Result::CanWinWith(win_choices)
    }
}
fn main() {
    let mut losses = 0;
    let mut wins = 0;
    let mut maybe_wins = 0;
    let mut games = 0;
    loop {
        games +=1;
        match play_one() {
            Result::AlwaysWin => {wins +=1;}
            Result::AlwaysLose => { losses +=1;}
            Result::CanWinWith(_) => {maybe_wins += 1;}
        }
        if games % 1000 == 0 {
            println!("Always win {}, Always lose {}, Can win {} out of {} games",wins,losses,maybe_wins, games);
        }
    }
}
