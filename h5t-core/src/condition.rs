use enumset::EnumSetType;
use std::{cmp::Ordering, num::NonZeroU32};

/// A condition and how long it lasts.
#[derive(Clone, Debug)]
pub struct Condition {
    /// The condition to apply.
    pub kind: ConditionKind,

    /// The duration of the condition.
    pub duration: ConditionDuration,
}

/// All possible conditions that can be applied to a combatant.
#[derive(EnumSetType, Debug, Hash)]
pub enum ConditionKind {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion,
    Frightened,
    Grappled,
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}

impl std::fmt::Display for ConditionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

macro_rules! make_abbreviations {
    ($($kind:ident => $abbreviation:expr),* $(,)?) => {
        impl ConditionKind {
            /// Returns an abbreviation for the condition that identifies it uniquely.
            ///
            /// The current abbreviations used are:
            ///
            $(
                #[doc = concat!(
                    "- [`",
                    stringify!($kind),
                    "`](ConditionKind::",
                    stringify!($kind),
                    "): `",
                    $abbreviation,
                    "`\n",
                )]
            )*
            pub fn abbreviation(self) -> &'static str {
                match self {
                    $(ConditionKind::$kind => $abbreviation,)*
                }
            }
        }
    };
}

make_abbreviations! {
    Blinded => "BL",
    Charmed => "CH",
    Deafened => "DE",
    Exhaustion => "EX",
    Frightened => "FR",
    Grappled => "GR",
    Incapacitated => "INC",
    Invisible => "INV",
    Paralyzed => "PA",
    Petrified => "PE",
    Poisoned => "PO",
    Prone => "PR",
    Restrained => "RE",
    Stunned => "ST",
    Unconscious => "UN",
}

/// Duration of a condition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ConditionDuration {
    /// The condition lasts until the end of the combatant's next turn.
    #[default]
    UntilNextTurn,

    /// The condition lasts for the given number of rounds.
    ///
    /// When the combatant's turn ends, the duration is decremented by one. When the duration
    /// is reduced to zero, the condition ends.
    Rounds(NonZeroU32),

    /// The condition lasts for the given number of minutes.
    ///
    /// One minute is equal to 10 rounds.
    Minutes(NonZeroU32),

    /// The condition lasts forever until it is manually removed.
    Forever,
}

impl std::fmt::Display for ConditionDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionDuration::UntilNextTurn => write!(f, "2 rounds"),
            ConditionDuration::Rounds(n) if n.get() == 1 => write!(f, "1 round"),
            ConditionDuration::Rounds(n) => write!(f, "{} rounds", n),
            ConditionDuration::Minutes(n) if n.get() == 1 => write!(f, "1 minute"),
            ConditionDuration::Minutes(n) => write!(f, "{} minutes", n),
            ConditionDuration::Forever => write!(f, "Forever"),
        }
    }
}

impl PartialOrd for ConditionDuration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConditionDuration {
    fn cmp(&self, other: &Self) -> Ordering {
        let (Some(rounds_left_self), Some(rounds_left_other)) = (
            self.rounds_left(),
            other.rounds_left(),
        ) else {
            return match (self, other) {
                (ConditionDuration::Forever, ConditionDuration::Forever) => Ordering::Equal,
                (ConditionDuration::Forever, _) => Ordering::Greater,
                (_, ConditionDuration::Forever) => Ordering::Less,
                _ => unreachable!(),
            };
        };

        rounds_left_self.cmp(&rounds_left_other)
    }
}

impl ConditionDuration {
    /// Returns the number of rounds until the condition ends. Returns [`None`] if the condition
    /// duration is [`ConditionDuration::Forever`].
    pub fn rounds_left(self) -> Option<u32> {
        match self {
            // NOTE: the value is 2, one turn to end the current turn and one to end the next
            ConditionDuration::UntilNextTurn => Some(2),
            ConditionDuration::Rounds(n) => Some(n.get()),
            ConditionDuration::Minutes(n) => Some(n.get() * 10),
            ConditionDuration::Forever => None,
        }
    }

    /// Returns a new [`ConditionDuration`] with one round subtracted.
    /// [`ConditionDuration::Forever`] simply returns itself. Returns [`None`] if the duration is
    /// or will be zero.
    pub fn decrement(self) -> Option<ConditionDuration> {
        if matches!(self, ConditionDuration::Forever) {
            return Some(ConditionDuration::Forever);
        }
        let rounds = self.rounds_left()?;
        Some(ConditionDuration::Rounds(NonZeroU32::new(rounds.checked_sub(1)?)?))
    }
}
