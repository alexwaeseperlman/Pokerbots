# Building a Bot

## Communicating with the engine
Bots get the game state and make their moves by communicating through stdin and stdout. 

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
    "run": "python3 rngbot.py"
}
```

We will run the command in `bot.json` and interact with that.