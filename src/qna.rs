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
const GROUPS: [Group; 110] = [
    Single(Query(&["😎", "🕶"], &["people in sunglasses", "sunglasses"])),
    Single(Query(&["🥳", "🎉", "🎊"], &["birthday party children", "people on a party"])),
    Single(Query(&["😤", "😠", "😡"], &["angry man", "angry woman"])),
    Single(Query(&["🤗", "👐", "👐🏻", "👐🏼"], &["hug", "hugging"])),
    Single(Query(&["🤫"], &["shushing man", "shushing woman"])),
    Single(Query(&["🤔"], &["thinking man"])),
    Single(Query(&["🫡"], &["saluting army man", "saluting army woman"])),
    Single(Query(&["😴", "💤", "🛌"], &["sleep"])),
    Single(Query(&["🥴", "🍺", "🍻", "🥂", "🥃"], &["drunk man", "drunk woman", "drinking alcohol"])),
    Single(Query(&["🤢", "🤮"], &["vomit in cartoon"])),
    Single(Query(&["🤧", "😷", "🤒", "🤕"], &["sick man"])),
    Single(Query(&["🤑", "💰", "💸", "💵"], &["money economics", "dollars", "euros"])),
    Single(Query(&["💥", "💣️"], &["bomb explodes cartoonish"])),
    Single(Query(&["💫", "🌠"], &["shootingstar"])),
    Single(Query(&["👌", "👌🏻", "👌🏼", "🆗"], &["ok hand"])),
    Single(Query(&["✌️", "✌🏻", "✌🏼"], &["showing victory hand"])),
    Single(Query(&["🤞", "🤞🏻", "🤞🏼"], &["showing crossed fingers"])),
    Single(Query(&["🤘", "🤘🏻", "🤘🏼"], &["people showing rock hand"])),
    Single(Query(&["✊", "✊🏻", "✊🏼"], &["raised fist"])),
    Single(Query(&["👏", "👏🏻", "👏🏼"], &["people clapping hands"])),
    Single(Query(&["🙌", "🙌🏻", "🙌🏼"], &["people raising hands up"])),
    Single(Query(&["🤝", "🤝🏻", "🤝🏼", "🫱🏻‍🫲🏼", "🫱🏻‍🫲🏽", "🫱🏼‍🫲🏻", ""], &["people handshake"])),
    Single(Query(&["🧠"], &["brain pictures"])),
    Single(Query(&["🦷"], &["tooth"])),
    Single(Query(&["🦴"], &["bone"])),
    Single(Query(&["👀", "👁️"], &["eyes"])),
    Single(Query(&["😛", "👅"], &["tongue"])),
    Single(Query(&["🙅", "🙅🏻", "🙅🏼", "🙅‍♂️", "🙅🏻‍♂️", "🙅🏼‍♂️", "🙅‍♀️", "🙅🏻‍♀️", "🙅🏼‍♀️", "❌"], &["people gesturing no crossing hands"])),
    Single(Query(&["🧑‍🎓", "🧑🏻‍🎓", "🧑🏼‍🎓", "👨‍🎓", "👨🏻‍🎓", "👨🏼‍🎓", "👩‍🎓", "👩🏻‍🎓", "👩🏼‍🎓"], &["students wearing graduate caps"])),
    Single(Query(&["🧑‍🍳", "🧑🏻‍🍳", "🧑🏼‍🍳", "👨‍🍳", "👨🏻‍🍳", "👨🏼‍🍳", "👩‍🍳", "👩🏻‍🍳", "👩🏼‍🍳"], &["professional cook wearing white with food"])),
    Single(Query(&["👷", "👷🏻", "👷🏼", "👷‍♂️", "👷🏻‍♂️", "👷🏼‍♂️", "👷‍♀️", "👷🏻‍♀️", "👷🏼‍♀️"], &["construction worker"])),
    Single(Query(&["🧙", "🧙🏻", "🧙🏼", "🧙‍♂️", "🧙🏻‍♂️", "🧙🏼‍♂️", "🧙‍♀️", "🧙🏻‍♀️", "🧙🏼‍♀️"], &["wizard"])),
    Single(Query(&["🧟", "🧟‍♂️", "🧟‍♀️"], &["zombie in cartoon"])),
    Single(Query(&["🏃", "🏃🏻", "🏃🏼", "🏃‍♂️", "🏃🏻‍♂️", "🏃🏼‍♂️", "🏃‍♀️", "🏃🏻‍♀️", "🏃🏼‍♀️"], &["running"])),
    Single(Query(&["⛹️", "⛹🏻", "⛹🏼", "⛹️‍♂️", "⛹🏻‍♂️", "⛹🏼‍♂️", "⛹️‍♀️", "⛹🏻‍♀️", "⛹🏼‍♀️"], &["playing basketball"])),
    Single(Query(&["🚴", "🚴🏻", "🚴🏼", "🚴‍♂️", "🚴🏻‍♂️", "🚴🏼‍♂️", "🚴‍♀️", "🚴🏻‍♀️", "🚴🏼‍♀️"], &["biking"])),
    Single(Query(&["🧘", "🧘🏻", "🧘🏼", "🧘‍♂️", "🧘🏻‍♂️", "🧘🏼‍♂️", "🧘‍♀️", "🧘🏻‍♀️", "🧘🏼‍♀️"], &["people in lotus position"])),
    Single(Query(&["🍌"], &["banana", "eat banana"])),
    Multiple(&[
        Query(&["🐶", "🐕️", "🦮", "🐩"], &["dog"]),
        Query(&["🐺"], &["wolf"]),
    ]),
    Single(Query(&["🦊"], &["fox"])),
    Multiple(&[
        Query(&["🙉", "🐵", "🐒"], &["monkey"]),
        Query(&["🦍"], &["gorilla"]),
        Query(&["🦧"], &["gorilla"]),
        ]),
    Single(Query(&["🐮", "🐄"], &["cow"])),
    Single(Query(&["🐷", "🐖"], &["pig"])),
    Multiple(&[
        Query(&["🐏"], &["ram"]),
        Query(&["🐑"], &["ewe"]),
    ]),
    Multiple(&[
        Query(&["🦁"], &["lion"]),
        Query(&["🐯"], &["tiger"]),
    ]),
    Single(Query(&["🐪", "🐫"], &["camel"])),
    Single(Query(&["🦒"], &["giraffe"])),
    Single(Query(&["🐘"], &["elephant"])),
    Single(Query(&["🐰", "🐇"], &["rabbit"])),
    Single(Query(&["🦔"], &["hedgehog"])),
    Single(Query(&["🐻"], &["bear"])),
    Single(Query(&["🐼"], &["panda"])),
    Single(Query(&["🦘"], &["kangaroo"])),
    Single(Query(&["🐔", "🐓"], &["chicken", "rooster"])),
    Single(Query(&["🐣", "🐤", "🐥"], &["baby chick"])),
    Single(Query(&["🐧"], &["penguin"])),
    Single(Query(&["🦆"], &["duck with green head"])),
    Single(Query(&["🦢"], &["swan"])),
    Single(Query(&["🦉"], &["owl"])),
    Single(Query(&["🦩"], &["flamingo"])),
    Single(Query(&["🦜"], &["parrot"])),
    Single(Query(&["🐸"], &["frog"])),
    Single(Query(&["🐊"], &["crocodile"])),
    Single(Query(&["🐢"], &["turtle"])),
    Multiple(&[
        Query(&["🐍"], &["snake"]),
        Query(&["🪱"], &["worm"]),
    ]),
    Single(Query(&["🦕", "🦖"], &["dinosaur"])),
    Multiple(&[
        Query(&["🦖"], &["dinosaur T-Rex"]),
        Query(&["🦕"], &["dinosaur sauropod"]),
    ]),
    Multiple(&[
        Query(&["🐳", "🐋"], &["whale", "spouting whale"]),
        Query(&["🐬"], &["dolphin"]),
        Query(&["🐟️"], &["fish"]),
        Query(&["🐠"], &["tropical fish"]),
        Query(&["🐡"], &["blowfish"]),
        Query(&["🦈"], &["shark"]),
    ]),
    Single(Query(&["🐙"], &["red octopus"])),
    Single(Query(&["🐌"], &["snail"])),
    Single(Query(&["🦋"], &["butterfly"])),
    Single(Query(&["🐝"], &["honeybee"])),
    Multiple(&[
        Query(&["🪲"], &["beetle"]),
        Query(&["🐞"], &["lady beetle"]),
    ]),
    Single(Query(&["🪳"], &["cockroach"])),
    Single(Query(&["🕸️"], &["spider web"])),
    Single(Query(&["🦂"], &["scorpion"])),
    Multiple(&[
        Query(&["🌸"], &["cherry blossom"]),
        Query(&["🪷"], &["pink lotus"]),
        Query(&["🌹"], &["rose"]),
        Query(&["🌺"], &["hibiscus"]),
        Query(&["🌻"], &["sunflower"]),
        Query(&["🌼"], &["blossom"]),
        Query(&["🌷"], &["tulip"]),
    ]),
    Single(Query(&["🌱"], &["seedling"])),
    Single(Query(&["🪴"], &["potted plant"])),
    Multiple(&[
        Query(&["🐭", "🐁"], &["mouse"]),
        Query(&["🐀"], &["rat"]),
        Query(&["🐹"], &["hamster"]),
    ]),
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
    Multiple(&[
        Query(&["💪", "💪🏻", "💪🏼"], &["strong man in gym", "strong woman in gym"]),
        Query(&["🏋️", "🏋🏻", "🏋🏼", "🏋️‍♂️", "🏋🏻‍♂️", "🏋🏼‍♂️", "🏋️‍♀️", "🏋🏻‍♀️", "🏋🏼‍♀️"], &["lifting weights"]),
    ]),
    Single(Query(&["🖕", "🖕🏻", "🖕🏼"], &["middle finger"])),
    Single(Query(&["✍️", "✍🏻", "✍🏼"], &["writing"])),
    Single(Query(&["🦶", "🦶🏻", "🦶🏼"], &["foot"])),
    Single(Query(&["👂", "👂🏻", "👂🏼"], &["ear"])),
    Single(Query(&["👃", "👃🏻", "👃🏼"], &["pictures of nose"])),
    Single(Query(&["👶", "👶🏻", "👶🏼"], &["child", "baby"])),
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
    Single(Query(&["🤦‍♂️", "🤦🏻‍♂️", "🤦🏼‍♂️", "🤦", "🤦🏻", "🤦🏼", "🤦‍♀️", "🤦🏻‍♀️", "🤦🏼‍♀️"], &["facepalm"])),
    Single(Query(&["🤷", "🤷🏻", "🤷🏼", "🤷‍♀️", "🤷🏻‍♀️", "🤷🏼‍♀️", "🤷‍♂️", "🤷🏻‍♂️", "🤷🏼‍♂️"], &["shrugging hands"])),
];
