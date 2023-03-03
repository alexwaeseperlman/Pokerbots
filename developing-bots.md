# Developing bots

## The game
Pokerbots play 2-player No Limit Texas Holdem' with fixed blinds. We chose this format 
instead of larger games or tournament style because it has fewer factors for bots to deal with.
This way you can focus on understanding your opponent and betting intelligently.

## How to play
At the start of the table the `button` is assigned to a random player, and the `button blind` is the other.
Before every round the `button` is forced to put the small blind into the pot (1),
and the `big blind` is forced to put the big blind (2).

At the end of each game, the `button` and `big blind` rotate.

TODO: EXPAND AND WRITE MORE.

## Actions
In a betting round you can raise, check, call or fold. They are represented as
specified:

|Action|Representation|
|----|----|
|Check|`X`|
|Call|`C`|
|Fold|`F`|
|Raise|`R<N>` (e.g. `R15` will raise by 15)|

Note that checking and calling are the same action. This is because a check
is equivalent to calling when you and your opponent have bet the same amount.

## Betting rounds
Before every action in a betting round, the engine will output a line containing 5
integers, `P`, `C_0`, `C_1`, `S_0`, `S_1` followed by a line for the player's action. `P` is the size of the pot from previous rounds; 
`C_0` is how many chips player `0` has bet this round; `C_1` is how many chips player `1` has bet this round;
`S_0` is the size of player `0`'s stack; `S_1` is the size of player`0`'s stack. The next line has an integer denoting the player followed by a character and potentially a number `N` denoting the action. After reading these lines, your bot must respond with an action. Calling will match your
opponent's bet for this betting round, raising will match their bet and then put down 
an additional `N`, and folding will immediately make you lose the round. A betting round ends
as soon as both players have made an action and a player calls.

Pre-flop, player `0` goes first. Post-flop in every other betting round, player `1` goes first.

## Representing cards
Cards are always 2 characters. The first represents value and the second represents suit. 
The value of a card is either represented as a single digit (for 2-9), or as the first letter of its name
(T=10, J=Jack, Q=Queen, K=King, A=Ace). The suit is represented as the first letter of its name 
(s=Spades, h=Hearts, s=Clubs, d=Diamonds).

For example:
`2d` represents the 2 of diamonds, and `Tc` represents the 10 of clubs.

## Communicating with the engine
Pokerbots get the game state and make their moves by communicating through stdin and stdout. Every round is given by
1. A line `2 B` to indicate the beginning of the betting round. 
2. A betting round.
3. A line `2 E` to indicate the end of the betting round.

This is what a round looks like:
1. A line `2 n` where `n = 0` or `1` to indicate your position. Player number `0` is always the button.
2. A line with two cards, which are your starting hand. 
3. A round where the first line of the betting round is `0 1 2 S_0 S_1`, and player `0` bets first. Note the `0 1 2` due to blinds.
7. One line containing 3 cards. This is this flop.
8. A round where the first line of the betting round is `P 0 0 S_0 S_1`, and player `1` bets first.
10. A line containing a single card. This is the turn.
11. A round where the first line of the betting round is `P 0 0 S_0 S_1`, and player `1` bets first.
12. A line containing a single card. This is the river.
13. There is a final round, and the game ends.
14. Your bot has the option to play another round against this opponent by outputting
`P`, or stop by outputting `Q`. 

TODO: EXPAND AND FIX UP.

## Example
The following is an example round for the pre-flop and the beginning of flop.
```
>> 2 0
>> Tc 5d
>> 2 B
>> 0 1 2 49 48
>> 0 R5
>> 0 6 2 44 48
<< R3
>> 1 R3
>> 0 6 9 44 41
>> 0 C
>> 0 9 9 41 41
>> 2 E
>> 3d 9d Qs
>> 2 B
>> 18 0 0 41 41
FINISH...
```
