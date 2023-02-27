# Developing bots

## The game
Pokerbots play 2-player Texas Holdem' with fixed blinds. We chose this format 
instead of larger games or tournament style because it has fewer factors for bots to deal with.
This way you can focus on understanding your opponent and betting intelligently.

## How to play
At the start of the game the 'button' is assigned to a random player.
Before every round the button is forced to put the small blind into the pot ($1),
and the other player is forced to put the big blind ($2). 

write this later ...

## Betting rounds
Before every action in a betting round, the engine will output a line containing 3
integers, `P`, `A`, and `B`. `P` is the size of the pot from previous rounds, 
`A` is how many chips player `0` has bet this round, 
and `B` is how many chips player `1` has bet this round. You are never explicitly
told your opponents action but you can infer it based on these values.
After reading this line your bot must respond with an action. Calling will match your
opponents bet for this betting round, raising will match their bet and then put down 
an additional `N`, and folding will immediately make you lose the round. A betting round ends
as soon as both players have made an action and a player calls.

In every betting round except for the first, player `1` goes first.

## Actions
In a betting round you can raise, check, call or fold. They are represented as
specified:

|Action|Representation|
|----|----|
|Check|`C`|
|Call|`C`|
|Fold|`F`|
|Raise|`R<N>` (e.g. `R15` will raise by $15)|

Note that checking and calling are the same action. This is because a check
is equivalent to calling when you and your opponent have bet the same amount.

## Communicating with the engine
Pokerbots get the game state and make their moves by communicating through stdin and stdout. 

This is what a round looks like:
1. A single line with an integer `0 <= N < = 1`. This is your player number.
Player number `0` is always the button.
2.  A line containing 2 integers representing the amount of chips in player `0` 
and player `1`'s stacks respectively.
4. A line containing two cards, which are your starting hand. 
5. A betting round, but due to blinds the first line is `0 1 2`, and player `0` bets first.
6. One line containing 3 cards. This is this flop.
7. A betting round
8. A line containing a single card. This is the turn.
9. A betting round
10. A line containing a single card. This is the river.
11. There is a final betting round and the game ends.
12. Your bot has the option to play another round against this opponent by outputting
`P`, or stop by outputting `Q`. If both players output `P` then there will be another round.

## Representing cards
Cards are always 2 characters. The first represents value and the second represents suit. 
The value of a card is either represented as a single digit (for 2-9), or as the first letter of its name
(T=10, J=Jack, Q=Queen, K=King, A=Ace). The suit is represented as the first letter of its name 
(S=Spades, H=Hearts, C=Clubs, D=Diamonds).

For example:
`2D` represents the 2 of diamonds, and `TC` represents the 10 of clubs.

## Programming a bot
Bots are uploaded as a zip containing a file named `bot.json`. This file contains a 
command to run it. For example:

```json
{
    "command": "python3 rngbot.py"
}
```

We will run the command in `bot.json` and interact with that.