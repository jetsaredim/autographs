export type ApprovedQuote = {
  id: string;
  quote: string;
  attribution: string;
  tone: string;
};

export const approvedQuotes = [
  {
    id: "star-wars-droids",
    quote: "These aren't the droids you're looking for.",
    attribution: "Obi-Wan Kenobi, Star Wars",
    tone: "sly misdirection",
  },
  {
    id: "last-crusade-x",
    quote: "X never, ever marks the spot.",
    attribution: "Indiana Jones, Indiana Jones and the Last Crusade",
    tone: "dry adventure",
  },
  {
    id: "emperors-new-groove-lever",
    quote: "Wrong lever!",
    attribution: "Yzma, The Emperor's New Groove",
    tone: "comic wrong turn",
  },
  {
    id: "fellowship-pass",
    quote: "You shall not pass!",
    attribution: "Gandalf, The Lord of the Rings: The Fellowship of the Ring",
    tone: "blocked path",
  },
  {
    id: "last-crusade-ticket",
    quote: "No ticket.",
    attribution: "Indiana Jones, Indiana Jones and the Last Crusade",
    tone: "terse rejection",
  },
  {
    id: "spaceballs-not-in-there",
    quote: "She's not in there!",
    attribution: "Dark Helmet, Spaceballs",
    tone: "empty-result gag",
  },
  {
    id: "empire-odds",
    quote: "Never tell me the odds!",
    attribution: "Han Solo, Star Wars: The Empire Strikes Back",
    tone: "stubborn optimism",
  },
  {
    id: "tolkien-wander",
    quote: "Not all those who wander are lost.",
    attribution: "J.R.R. Tolkien, The Fellowship of the Ring",
    tone: "warm reassurance",
  },
  {
    id: "empire-try",
    quote: "Do... or do not. There is no try.",
    attribution: "Yoda, Star Wars: The Empire Strikes Back",
    tone: "gentle resolve",
  },
  {
    id: "star-wars-bad-feeling",
    quote: "I have a bad feeling about this.",
    attribution: "Luke Skywalker, Star Wars",
    tone: "ominous discovery",
  },
  {
    id: "jaws-bigger-boat",
    quote: "You're gonna need a bigger boat.",
    attribution: "Chief Brody, Jaws",
    tone: "the scope just changed",
  },
  {
    id: "wizard-home",
    quote: "There's no place like home.",
    attribution: "Dorothy Gale, The Wizard of Oz",
    tone: "return path",
  },
  {
    id: "finding-nemo-swimming",
    quote: "Just keep swimming.",
    attribution: "Dory, Finding Nemo",
    tone: "cheerful persistence",
  },
  {
    id: "raiders-snakes",
    quote: "Why did it have to be snakes?",
    attribution: "Indiana Jones, Raiders of the Lost Ark",
    tone: "reluctant obstacle",
  },
  {
    id: "apollo-problem",
    quote: "Houston, we have a problem.",
    attribution: "Jim Lovell, Apollo 13",
    tone: "calm escalation",
  },
  {
    id: "return-jedi-trap",
    quote: "It's a trap!",
    attribution: "Admiral Ackbar, Star Wars: Return of the Jedi",
    tone: "sudden warning",
  },
  {
    id: "wizard-yellow-brick-road",
    quote: "Follow the yellow brick road.",
    attribution: "The Munchkins, The Wizard of Oz",
    tone: "clear next step",
  },
  {
    id: "maltese-falcon-dreams",
    quote: "The stuff that dreams are made of.",
    attribution: "Sam Spade, The Maltese Falcon",
    tone: "wistful artifact",
  },
  {
    id: "some-like-it-hot-perfect",
    quote: "Well, nobody's perfect.",
    attribution: "Osgood Fielding III, Some Like It Hot",
    tone: "forgiving shrug",
  },
  {
    id: "toy-story-infinity",
    quote: "To infinity and beyond!",
    attribution: "Buzz Lightyear, Toy Story",
    tone: "buoyant onward",
  },
] as const satisfies readonly ApprovedQuote[];

export const selectApprovedQuote = (seed = Date.now()): ApprovedQuote => {
  const index = Math.abs(Math.trunc(seed)) % approvedQuotes.length;
  return approvedQuotes[index];
};
