# Scoring

Scoring is done using the Elo rating system. We calculate the expected score for each player and then update their ratings based on the actual score. The expected score is calculated using the following formula:

$$E_i = \frac{1}{1 + 10^{(R_j - R_i)/400}}$$ (see [get_rating_change](https://github.com/alexwaeseperlman/Pokerbots/blob/5ea0dd9636e93cf5b9379709ea97cd3715bda818/workers/results/src/rating.rs#L14))

Where $R_i$ and $R_j$ are the ratings of the two players and $E$ is the expected score for player $i$. The actual score is determined by the proportion of the total number
of chips in the game that each player has at the end of the game. 
For example in a game where both players start with 500 chips, if at the end
player 1 has 800 and player 2 has 200, then player 1's score is 0.8 and player 2's score is 0.2.

The rating change for each player is then calculated using the following formula:

$$\Delta R_i = K(S_i - E_i)$$ (see [get_rating_change](https://github.com/alexwaeseperlman/Pokerbots/blob/5ea0dd9636e93cf5b9379709ea97cd3715bda818/workers/results/src/rating.rs#L16))

Where $S_i$ is the actual score for player $i$, $E_i$ is the expected score for player $i$, and $K$ is a constant that determines the maximum rating change. In our case, we use $K = 12$ (which is subject to change).