import React, { useCallback, useEffect, useState } from "react";
import { apiUrl } from "../state";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Slider, Typography, Input } from "@mui/joy";
import GameCard from "./GameCard";
import { Game } from "@bindings/Game";

interface GameState {
  game: string,
  step: number,
  challenger_stack: number,
  defender_stack: number,
  challenger_pushed: number,
  defender_pushed: number,
  pot: number,
  challenger_hand: string,
  defender_hand: string,
  flop: string,
  turn: string,
  river: string,
  button: string,
  round: string,
  last_action: string,
  sb: string,
}

interface card {
  rank: string;
  suit: string;
}

function stringToCard(cardStr: string): card {
  if (cardStr.length !== 2) {
    console.error('Invalid card string format. Must be 2 characters.');
    return { rank: "", suit: "" };
  }
  const fixTen = (s: string) => {
    if (s == "T") {
      return "10";
    }
    else {
      return s;
    }
  }
  const rankChar = fixTen(cardStr.charAt(0));
  const suitChar = cardStr.charAt(1);

  const rank = rankChar.toUpperCase();
  const suit = suitChar.toLowerCase();

  if (!rank || (suit !== 's' && suit !== 'h' && suit !== 'd' && suit !== 'c')) {
    console.error('Invalid card string values.');
    return { rank: "", suit: "" };
  }
  const abrToFullName = (suit: string) => {
    switch (suit) {
      case 'h':
        return 'hearts';
      case 'd':
        return 'diamonds';
      case 'c':
        return 'clubs';
      case 's':
        return 'spades';
      default:
        return '';
    }
  };

  return { rank, suit: abrToFullName(suit) };
}


const GameTable: React.FC<GameState> = ({ challenger_hand, defender_hand, flop, turn, river }) => {
  return [
    <div className="cards"> {challenger_hand ? challenger_hand.split(" ").map(c => {
      return (<GameCard card={stringToCard(c)} />);
    }) : <GameCard card={{ rank: "", suit: "" }} />}
    </div>,

    <div className="cards">
      {flop ? flop.split(" ").map(c => {
        return <GameCard card={stringToCard(c)} />;
      })
        :
        [<GameCard card={{ rank: "", suit: "" }} />,
        <GameCard card={{ rank: "", suit: "" }} />,
        <GameCard card={{ rank: "", suit: "" }} />]}

      {turn ? turn.split(" ").map(c => {
        return (<GameCard card={stringToCard(c)} />);
      }) : <GameCard card={{ rank: "", suit: "" }} />}

      {river ? river.split(" ").map(c => {
        return (<GameCard card={stringToCard(c)} />);
      }) : <GameCard card={{ rank: "", suit: "" }} />}

    </div>,
    <div className="cards"> {defender_hand ? defender_hand.split(" ").map(c => {
      return (<GameCard card={stringToCard(c)} />);
    }) : <GameCard card={{ rank: "", suit: "" }} />}
    </div>,
  ]
}
const ProfileCards: React.FC<GameState> = ({ challenger_stack, defender_stack, challenger_pushed, defender_pushed, sb }) => {
  return <div className="bots-information" >
    <div className="bot top">
      <span> Challenger </span>
      <span> Stack: {challenger_stack}</span>
      <span> Pushed: {challenger_pushed}</span>
    </div>
    <div className="bot bottom">
      <span> Defender </span>
      <span> Stack: {defender_stack}</span>
      <span> Pushed: {defender_pushed}</span>
    </div>
  </div>
}


function GetGameState({ gameId, step }: { gameId: string, step: number; }) {

  const [logs, setLogs] = useState<GameState>()

  const fetchData = () => {

    fetch(
      `${apiUrl}/game-record?id=${gameId}&round=${step}`)
      .then(res => { return res.json() })
      .then(data => { setLogs(data) }
      )
  }

  useEffect(() => fetchData(), [step])
  console.log(logs);
  return (
    logs ?
      <div className="tricol">
        <div className="col">
          <ProfileCards {...logs} />
        </div>
        <div className="col">
          <GameTable {...logs} />
        </div>
        <div className="col">
        </div>
      </div>
      : "");
}

export default function GameVisualizer({
  gameId,
}: {
  gameId: string;
}) {
  const [max, setMax] = useState(0)
  const [inputValue, setInputValue] = useState(0);
  const handleInputChange = (event: any) => {
    setInputValue(parseInt(event.target.value) || 0);
  }
  useEffect(
    () => {
      if (max == 0)
        fetch(`${apiUrl}/game-length?game_id=${gameId}`).then(data => data.text()).then(data => setMax(parseInt(data)));
    },
    [max],
  )
  return (
    <Box className="game">
      <Card className="inner" sx={{ p: 2, flexGrow: 1, maxWidth: "100hv", maxHeight: "100%", mb: 2 }}>
        <Typography level="h3" mb={2}>
          Game {gameId}
        </Typography>
        <Box
          sx={{
            width: "100%",
            overflow: "hidden",
          }}
        >
        </Box>
        <GetGameState gameId={gameId} step={inputValue} />
        <Input type="number" value={inputValue} onChange={handleInputChange} sx={{ width: "10%" }} slotProps={{
          input: {
            min: 0,
            max: max,
          },
        }} />
        <Slider
          value={inputValue}
          onChange={handleInputChange}
          step={1}
          marks
          min={0}
          max={max} />
      </Card>
    </Box>
  );
}
