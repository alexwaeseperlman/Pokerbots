import pandas as pd
import sys
df = pd.read_csv('hands.csv')

def card_val(card):
    if card[0] == 'A': return 1
    elif card[0] == 'T': return 10
    elif card[0] == 'J': return 11
    elif card[0] == 'Q': return 12
    elif card[0] == 'K': return 13
    return int(card[0])

while True:
    line = input().split()
    assert(line[0] == 'START')
    position = line[1]

    state = -1

    hand = []
    cards = []

    p = 0.0

    while line[0] != 'END':
        if line[0] == 'STACK':
            pushed, stack, opPushed, opStack = [int(i) for i in line[1:]]
            # Act
            # maximize p log (1+x) + (1-p) log (1-x)
            target = int((2*p-1)*stack)
            if position == 'SB' and target <= 1: print('F')
            else: print(f'R{max(target-pushed, 0)}', flush=True)
        elif line[0] == 'PREFLOP':
            hand = line[1:]
            state = 0

            hand_vals = sorted([card_val(hand[0]), card_val(hand[1])])
            suited = int(hand[0][1] == hand[1][1])

            row = df[(df['lo'] == hand_vals[0]) & (df['hi'] == hand_vals[1]) & (df['suited'] == suited)].iloc[0]
            p = row['win']/(row['win'] + row['loss'])
        elif line[0] == 'FLOP':
            cards = line[1:]
            state = 1
        elif line[0] == 'TURN':
            cards.append(line[1])
            state = 2
        elif line[0] == 'RIVER':
            cards.append(line[1])
            state = 3

        line = input().split()
    