# Developing bots

## The game
Bots play 2-player Texas Holdem' with fixed blinds. We chose this format 
instead of larger games or tournament style because it has fewer factors for bots to deal with.
This way you can focus on understanding your opponent and betting intelligently.

## How to play
At the start of the game the 'button' is assigned to a random player.
Before every round the button is forced to put the small blind into the pot ($1),
and the other player is forced to put the big blind ($2). 

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