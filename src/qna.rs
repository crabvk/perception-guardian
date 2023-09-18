use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use std::fmt;
use Group::*;

struct Query(&'static [&'static str], &'static [&'static str]);

impl Query {
    fn pick_emoji(&self, rng: &mut ThreadRng) -> &'static str {
        if self.0.len() == 1 {
            self.0[0]
        } else {
            self.0.choose(rng).unwrap()
        }
    }

    fn pick_phrase(&self, rng: &mut ThreadRng) -> &'static str {
        if self.1.len() == 1 {
            self.1[0]
        } else {
            self.1.choose(rng).unwrap()
        }
    }
}

pub struct Combination {
    pub emojis: Box<[&'static str]>,
    pub answer: &'static str,
    pub query_phrase: &'static str,
}

impl Combination {
    pub fn pick(queries_amount: usize) -> Combination {
        let mut rng = thread_rng();
        let queries: Box<[&Query]> = GROUPS
            .choose_multiple(&mut rng, queries_amount)
            .map(|group| match group {
                Single(query) => query,
                Multiple(queries) => queries.choose(&mut rng).unwrap(),
            })
            .collect();
        let emojis: Box<[&str]> = queries
            .iter()
            .map(|query| query.pick_emoji(&mut rng))
            .collect();
        let answer_idx = rng.gen_range(0..queries_amount);
        let query_phrase = queries[answer_idx].pick_phrase(&mut rng);

        Combination {
            answer: &emojis[answer_idx],
            emojis,
            query_phrase,
        }
    }
}

impl fmt::Display for Combination {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Variants: {:?}, Answer: \"{}\", Query phrase: \"{}\"",
            self.emojis, self.answer, self.query_phrase
        )
    }
}

enum Group {
    Single(Query),
    Multiple(&'static [Query]),
}

#[rustfmt::skip]
const GROUPS: [Group; 39] = [
    Single(Query(&["😎", "🕶"], &["people in sunglasses", "sunglasses"])),
    Single(Query(&["🥳", "🎉", "🎊"], &["birthday party children", "people on a party"])),
    Single(Query(&["😤", "😠", "😡"], &["angry man", "angry woman"])),
    Single(Query(&["🤗"], &["hug", "hugging"])),
    Single(Query(&["🤔"], &["thinking man"])),
    Single(Query(&["🥴", "🍺", "🍻", "🥂", "🥃"], &["drunk man", "drunk woman", "drinking alcohol"])),
    Single(Query(&["🤢", "🤮"], &["vomit in cartoon"])),
    Single(Query(&["🤧", "😷", "🤒", "🤕"], &["sick man"])),
    Single(Query(&["🤑", "💰", "💸", "💵"], &["money economics", "dollars", "euros"])),
    Multiple(&[
        Query(&["🤠"], &["cowboy"]),
        Query(&["🐴", "🐎"], &["horse"]),
    ]),
    Single(Query(&["😈", "👹"], &["devil"])),
    Single(Query(&["🤡"], &["clown"])),
    Single(Query(&["💩"], &["poop in cartoon"])),
    Single(Query(&["👻"], &["ghost"])),
    Single(Query(&["💀", "☠️"], &["skull"])),
    Single(Query(&["👽"], &["alien", "ufo"])),
    Single(Query(&["🤖"], &["robot"])),
    Single(Query(&["🎃"], &["halloween", "pumpkin"])),
    Single(Query(&["😺", "🐈"], &["cat", "kitty"])),
    Single(Query(&["🤝"], &["handshake"])),
    Single(Query(&["👍", "👍🏻", "👍🏼"], &["people thumbs up"])),
    Single(Query(&["👎", "👎🏻", "👎🏼"], &["people thumbs down sad"])),
    Single(Query(&["💪", "💪🏻", "💪🏼"], &["strong man in gym", "strong woman in gym"])),
    Single(Query(&["🖕", "🖕🏻", "🖕🏼"], &["middle finger"])),
    Single(Query(&["✍️", "✍🏻", "✍🏼"], &["writing"])),
    Single(Query(&["🦶", "🦶🏻", "🦶🏼"], &["foot"])),
    Single(Query(&["👂", "👂🏻", "👂🏼"], &["ear"])),
    Single(Query(&["👃", "👃🏻", "👃🏼"], &["pictures of nose"])),
    Single(Query(&["👶", "👶🏻", "👶🏼"], &["child"])),
    Multiple(&[
        Query(&["🧔🏻‍♀️", "🧔", "🧔🏻", "🧔‍♂️", "🧔🏻‍♂️"], &["beard"]),
        Query(&["👴", "👴🏻", "👴🏼"], &["old man"]),
        Query(&["👵", "👵🏻", "👵🏼", "🧓", "🧓🏻", "🧓🏼"], &["old lady"]),
    ]),
    Single(Query(&["👮‍♀️", "👮🏻‍♀️", "👮🏼‍♀️", "👮", "👮🏻", "👮🏼", "👮‍♂️", "👮🏻‍♂️", "👮🏼‍♂️"], &["police uniform"])),
    Single(Query(&["👩‍💻", "👩🏻‍💻", "👩🏼‍💻", "🧑‍💻", "🧑🏻‍💻", "🧑🏼‍💻", "👨‍💻", "👨🏻‍💻", "👨🏼‍💻"], &["programmer with computer"])),
    Single(Query(&["👩‍🚒", "👩🏻‍🚒", "👩🏼‍🚒", "🧑‍🚒", "🧑🏻‍🚒", "🧑🏼‍🚒", "👨‍🚒", "👨🏻‍🚒", "👨🏼‍🚒"], &["fireman at work"])),
    Single(Query(&["👩‍🚀", "👩🏻‍🚀", "👩🏼‍🚀", "🧑‍🚀", "🧑🏻‍🚀", "🧑🏼‍🚀", "👨‍🚀", "👨🏻‍🚀", "👨🏼‍🚀"], &["spaceman"])),
    Single(Query(&["👰‍♀️", "👰🏻‍♀️", "👰🏼‍♀️", "👰", "👰🏻", "👰🏼"], &["wife in wedding dress"])),
    Single(Query(&["🤴", "🤴🏻", "🤴🏼", "👑"], &["king with a crown", "crown"])),
    Single(Query(&["🎅", "🎅🏻", "🎅🏼"], &["santa clause"])),
    Single(Query(&["🤦‍♂️", "🤦🏻‍♂️", "🤦🏼‍♂️"], &["facepalm"])),
    Single(Query(&["🤷‍♀️", "🤷🏻‍♀️", "🤷🏼‍♀️", "🤷‍♂️", "🤷🏻‍♂️", "🤷🏼‍♂️"], &["shrugging hands"])),
];
