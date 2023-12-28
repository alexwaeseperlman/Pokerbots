import React, { useCallback, useEffect, useRef, useState } from "react";
import { apiUrl } from "../state";
import Box from "@mui/joy/Box";
import {
  Card,
  Grid,
  Sheet,
  Slider,
  Typography,
  Input,
  CircularProgress,
} from "@mui/joy";
import GameCard from "./GameCard";
import { Game } from "@bindings/Game";
import HeaderFooter from "../components/HeaderFooter";
import { GameStateSQL as GameState } from "@bindings/GameStateSQL";
import { TeamBotStack } from "../components/Tables/GameList/BotCard";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";
import { TeamStatusStack } from "../components/Tables/GameList/TeamStatusStack";
import { KeyValue } from "../components/KeyValue";
import { relative } from "path";

interface Card {
  rank: string;
  suit: string;
}

function stringToCard(cardStr: string): Card {
  if (cardStr.length !== 2) {
    console.error("Invalid card string format. Must be 2 characters.");
    return { rank: "", suit: "" };
  }
  const fixTen = (s: string) => {
    if (s == "T") {
      return "10";
    } else {
      return s;
    }
  };
  const rankChar = fixTen(cardStr.charAt(0));
  const suitChar = cardStr.charAt(1);

  const rank = rankChar.toUpperCase();
  const suit = suitChar.toLowerCase();

  if (!rank || (suit !== "s" && suit !== "h" && suit !== "d" && suit !== "c")) {
    console.error("Invalid card string values.");
    return { rank: "", suit: "" };
  }
  const abrToFullName = (suit: string) => {
    switch (suit) {
      case "h":
        return "hearts";
      case "d":
        return "diamonds";
      case "c":
        return "clubs";
      case "s":
        return "spades";
      default:
        return "";
    }
  };

  return { rank, suit: abrToFullName(suit) };
}

function GameTable({
  challenger_hand,
  defender_hand,
  flop,
  turn,
  river,
}: GameState) {
  return [
    <div className="cards">
      {" "}
      {challenger_hand ? (
        challenger_hand.split(" ").map((c) => {
          return <GameCard card={stringToCard(c)} />;
        })
      ) : (
        <GameCard card={{ rank: "", suit: "" }} />
      )}
    </div>,

    <div className="cards">
      {flop
        ? flop.split(" ").map((c) => {
            return <GameCard card={stringToCard(c)} />;
          })
        : [
            <GameCard card={{ rank: "", suit: "" }} />,
            <GameCard card={{ rank: "", suit: "" }} />,
            <GameCard card={{ rank: "", suit: "" }} />,
          ]}

      {turn ? (
        turn.split(" ").map((c) => {
          return <GameCard card={stringToCard(c)} />;
        })
      ) : (
        <GameCard card={{ rank: "", suit: "" }} />
      )}

      {river ? (
        river.split(" ").map((c) => {
          return <GameCard card={stringToCard(c)} />;
        })
      ) : (
        <GameCard card={{ rank: "", suit: "" }} />
      )}
    </div>,
    <div className="cards">
      {" "}
      {defender_hand ? (
        defender_hand.split(" ").map((c) => {
          return <GameCard card={stringToCard(c)} />;
        })
      ) : (
        <GameCard card={{ rank: "", suit: "" }} />
      )}
    </div>,
  ];
}

function GetGameState({
  gameId,
  step,
  game,
}: {
  gameId: string;
  step: number;
  game: GameWithBotsWithResult<BotWithTeam<Team>>;
}) {
  const [game_state, setGameState] = useState<GameState>();
  const [logs, setLogs] = useState<string>("");

  const fetchData = () => {
    fetch(`${apiUrl}/game-record?id=${gameId}&round=${step}`)
      .then((res) => {
        return res.json();
      })
      .then((data: GameState) => {
        setGameState(data);
      });
    fetch(`${apiUrl}/game-log?id=${gameId}`)
      .then((body) => body.text())
      .then((text) => setLogs(text));
  };

  const actionNote = (
    <Typography level="h3" color="inherit">
      {JSON.stringify(game_state?.action_val)}
    </Typography>
  );

  useEffect(() => fetchData(), [step]);
  console.log("state", game_state);

  return game_state ? (
    <Box
      sx={(theme) => ({
        display: "grid",
        gridTemplateRows: "repeat(3, auto)",
        gridTemplateAreas: '"challenger" "game" "defender"',
        [theme.breakpoints.up("md")]: {
          gridTemplateColumns: "auto 1fr auto",
          gridTemplateAreas: '"challenger game defender"',
          gridTemplateRows: "auto",
        },
        flexGrow: 1,
      })}
    >
      <Box
        sx={{
          display: "flex",
          gridArea: "challenger",
          flexDirection: "column",
          gap: 2,
        }}
      >
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
          }}
        >
          <TeamStatusStack
            direction="Challenger"
            bot={game.challenger}
            size="large"
            error={game.result?.error_type}
            rating={game.result?.challenger_rating}
            ratingChange={game.result?.challenger_rating_change}
          />
        </Box>
        <KeyValue
          keyName="Pushed"
          value={`${game_state.challenger_pushed}/${game_state.challenger_stack}`}
        />
        <KeyValue
          keyName="Position"
          value={<>{game_state.sb == "Challenger" ? "SB" : "BB"}</>}
        />
        <Box>{game_state.action_time}</Box>
      </Box>
      <Box
        sx={{
          gridArea: "game",
          position: "relative",
          minHeight: "350px",
        }}
      >
        <Box
          sx={{
            position: "absolute",
            height: "100%",
            display: "flex",
            left: 0,
            right: 0,
            flexDirection: "column",
            justifyContent: "center",
            alignItems: "center",
            gap: 4,
          }}
        >
          <Typography level="h3" color="inherit">
            Pot {game_state.defender_pushed + game_state.challenger_pushed}
          </Typography>
          <GameTable {...game_state} />
        </Box>
      </Box>
      <Box
        sx={{
          display: "flex",
          gridArea: "defender",
          flexDirection: "column",
          justifyContent: "flex-end",
          gap: 2,
        }}
      >
        <Box
          sx={{
            flexDirection: "row",
            display: "flex",
            alignItems: "center",
          }}
        >
          <TeamStatusStack
            direction="Defender"
            bot={game.defender}
            size="large"
            error={game.result?.error_type}
            rating={game.result?.defender_rating}
            ratingChange={game.result?.defender_rating_change}
          />
        </Box>
        <KeyValue
          keyName="Pushed"
          value={
            <>
              {game_state.defender_pushed}/{game_state.defender_stack} (
              {actionNote})
            </>
          }
        />
        <KeyValue
          keyName="Position"
          value={<>{game_state.sb == "Defender" ? "SB" : "BB"}</>}
        />
      </Box>
    </Box>
  ) : (
    <CircularProgress />
  );
}

export default function GameVisualizer({ gameId }: { gameId: string }) {
  const [max, setMax] = useState(0);
  const [inputValue, setInputValue] = useState(0);
  const [game, setGame] = useState<GameWithBotsWithResult<BotWithTeam<Team>>>();

  const handleInputChange = (event: any) => {
    setInputValue(parseInt(event.target.value) || 0);
  };
  useEffect(() => {
    if (max == 0)
      fetch(`${apiUrl}/game-length?game_id=${gameId}`)
        .then((data) => data.text())
        .then((data) => setMax(parseInt(data)));
  }, [max]);

  useEffect(() => {
    fetch(`${apiUrl}/games?id=${gameId}&page=0&page_size=1`)
      .then((res) => res.json())
      .then((data) => {
        setGame(data[0]);
      });
  }, [gameId]);

  if (!game || isNaN(max)) {
    return (
      <HeaderFooter fullWidth>
        <Box
          sx={{
            gridArea: "content",
            display: "flex",
            justifyContent: "center",
            alignItems: "center",
          }}
        >
          <CircularProgress />
        </Box>
      </HeaderFooter>
    );
  }
  return (
    <HeaderFooter fullWidth>
      <Box
        sx={{
          gridArea: "head",
        }}
      >
        <Typography
          level="h3"
          mb={2}
          sx={{
            overflowWrap: "anywhere",
          }}
          color='inherit'
        >
          Game {gameId}
        </Typography>
      </Box>
      <Box
        className="game"
        sx={{
          gridArea: "content",
          display: "flex",
          height: "100%",
          flexDirection: "column",
          alignItems: "stretch",
        }}
      >
        <GetGameState gameId={gameId} step={inputValue} game={game} />
        <Input
          type="number"
          sx={{
            maxWidth: "150px",
          }}
          value={inputValue}
          onChange={handleInputChange}
          slotProps={{
            input: {
              min: 0,
              max: max,
            },
          }}
        />
        <Slider
          value={inputValue}
          onChange={handleInputChange}
          step={1}
          marks
          min={0}
          max={max}
        />
      </Box>
    </HeaderFooter>
  );
}