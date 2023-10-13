# Building a Bot


## The game
Bots in UPAC play a version of No Limit Holdem. Games are always played between two bots.
One is labelled the challenger, and the other labelled the defender. These labels
don't mean much for now, but they might become relevant in the future. Games are played
for 100 rounds, or until one bot runs out of chips. Bots start with 50 chips,
so the total number of chips in a game is 100. The number of chips you end with
is the 'score', used to calculate your bot's elo change.

## Communicating with the engine
Bots get the game state and make their moves by communicating through stdin and stdout. 
The stdout and stdin of each bot is stored in the 'public log' for a game, and the stderr
for each bot is stored in the 'defender log' or 'challenger log'

This is the structure of a round. I will assume that you know the rules for heads up no limit holdem.
1. `START (SB|BB)`, where `START SB` means you're the small blind for this round,
   and `START BB` means you're the big blind for this round.
2. `PREFLOP (Card) (Card)`. These two cards are your hole cards. The other player cannot see them.
3. The engine enters a betting round. See the following section for
   an explanation.
4. `FLOP (Card) (Card) (Card)`. These are three shared cards. Both players see the same cards.
5. A betting round
6. `TURN (Card)`. This is shared card.
7. A betting round
8.  `RIVER (Card)`. This is a shared card
9. A betting round
10. `END SHOWDOWN (TIE (Card) (Card)|WINNER (SB|BB) (HIDDEN|SHOWN (Card) (Card)))`. 
    This means the round ended by going to a showdown. 
    If the showdown was a tie, then the engine prints `END SHOWDOWN TIE (Card) (Card)`, 
    where the two cards are the opponents hole cards.
    Else if the showdown was not a tie, the engine outputs a winner.
    The engine always outputs `END SHOWDOWN WINNER (SB|BB) SHOWN (Card) (Card)` to the loser, since the winner
    must show their cards (the printed cards are the ones belonging to the opposing player). 
    If the loser was the last aggressor (last person to raise by a positive amount) 
    then they must show their hand to the winner too. 
    Otherwise the winner receives `END SHOWDOWN WINNER (SB|BB) HIDDEN`

## Betting rounds
Betting rounds are where bots get to act. They are structured like this:

1. `STACK (a) (b) (c) (d)`. This message is sent to you when it's time to make an action.
   it contains four non-negative integers between 0 and 50. `a` is the number of chips 
   that your bot has pushed in this round. `b` is the total number of chips in your stack
   (including the amount that you pushed in this round). And `c` and `d` are these quantities
   but for the opposing player
2. Your bot gives an action, either outputting `F`, `C`, or `R<n>` (`<n>` is an integer, e.g. `R6`).
   These actions respectively represent "Fold", "Call", or "Raise by `n`". 
   Raising will first match the opponent's bet, and then push an additional `n` chips. 
   Note that this means that `C` is equivalent `R0`.
3. If a player folds, then both players receive `END FOLD (SB|BB)`, where `END FOLD SB` means that
   the small blind folded and `END FOLD BB` means that the big blind folded.
4. Once both players have made an action in this betting round, 
   and they have both pushed the same amount, the betting round ends.

## Representing cards
Cards are always 2 characters. The first represents value and the second represents suit. 
The value of a card is either represented as a single digit (for 2-9), or as the first letter of its name
(T=10, J=Jack, Q=Queen, K=King, A=Ace). The suit is represented as the first letter of its name 
(s=Spades, h=Hearts, c=Clubs, d=Diamonds).

For example:
`2d` represents the 2 of diamonds, and `Tc` represents the 10 of clubs.

## Programming a bot
Bots are uploaded as a zipped folder containing a file named `bot.json`. This file contains a name,
a build command, and a run command. For example:

```json
{
    "name": "FOY",
    "build": "g++ bot.cpp -O2 -o bot",
    "run": "./bot"
}
```

Right now your zip file must be created by zipping a folder named `bot`. When you upload a zip file,
a build event gets queued. The bot is then built in an environment where it has write access.
The built bot is then zipped, uploaded, and a test game is queued to verify that there aren't
any simple errors with running the bot. After the test game is finished, you
will be able to make this bot active so it will play games for you.

## Matchmaking
Every few seconds a new game is queued for each team with an active bot. 
The game is always queued with a bot of similar rating. After a game is finished,
the final chip values divided by 100 are used as the score in an elo calculation.
If a bot has a runtime error during the game then its score is considered to be zero,
and the other bot gets 1
(this is harsh but necessary to prevent bots from self-destructing when they are losing).
