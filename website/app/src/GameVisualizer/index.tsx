import React, { useCallback, useEffect, useRef, useState } from "react";
import { apiUrl } from "../state";
import Box from "@mui/joy/Box";
import {
  Grid,
  Sheet,
  Slider,
  Typography,
  Input,
  CircularProgress,
  IconButton,
  ButtonGroup,
} from "@mui/joy";
import GameCard from "./GameCard";
import { Game } from "@bindings/Game";
import HeaderFooter from "../components/HeaderFooter";
import { GameStateSQL as GameState } from "@bindings/GameStateSQL";
import { TeamBotStack } from "../components/Tables/GameList/BotCard";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { Card } from "@bindings/Card";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";
import { TeamStatusStack } from "../components/Tables/GameList/TeamStatusStack";
import { KeyValue } from "../components/KeyValue";
import { relative } from "path";
import { Action } from "@bindings/Action";
import { useParams, useSearchParams } from "react-router-dom";
import { Pause, PlayArrow, SkipNext, SkipPrevious } from "@mui/icons-material";
import bgImage from "./bg.png";
import { EndReason } from "@bindings/EndReason";

function roundName(cardCount: number) {
  if (cardCount == 0) return "Pre-flop";
  if (cardCount == 3) return "Flop";
  if (cardCount == 4) return "Turn";
  if (cardCount == 5) return "River";
}

function GameTable({
  challenger_hand,
  defender_hand,
  community_cards,
}: GameState) {
  const allCards: (Card | undefined)[] = community_cards.slice();
  while (allCards.length < 5) allCards.push(undefined);
  console.log(defender_hand, challenger_hand, allCards, community_cards);
  return [
    <div className="cards">
      {allCards.slice(0, 3).map((c) => {
        return <GameCard card={c} />;
      })}
    </div>,
    <div className="cards">
      {allCards.slice(3, 5).map((c) => {
        return <GameCard card={c} />;
      })}
    </div>,
  ];
}

function ActionNote({ action }: { action: Action | undefined }) {
  if (!action) return <Typography color="inherit">Did not act</Typography>;
  else if (action == "Fold") {
    return <Typography color="inherit">Fold</Typography>;
  } else if (action.Raise == 0) {
    return <Typography color="inherit">Check/call</Typography>;
  } else {
    return <Typography color="inherit">Raise {action.Raise}</Typography>;
  }
}

function endMessage(endReason: EndReason) {
  if (endReason == "Tie") {
    return "Tie";
  }
  if ("WonShowdown" in endReason) {
    return `${endReason.WonShowdown} won by showdown`;
  }

  if ("LastToAct" in endReason) {
    return `${endReason.LastToAct} won by fold`;
  }
}

function Status({
  hand,
  pushed,
  position,
  lastAction,
  stack
}: {
  hand: Card[];
  pushed: number;
  stack: number
  position: "SB" | "BB";
  lastAction: Action | undefined;
}) {
  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "flex-start",
          gap: 1,
        }}
      >
        <KeyValue keyName="Pushed" value={`${pushed}/${stack}`} />
        <KeyValue keyName="Position" value={position} />
        <KeyValue
          keyName="Last Action"
          value={<ActionNote action={lastAction} />}
        />
      </Box>
      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
          gap: 1,
        }}
      >
        {hand.map((c) => (
          <GameCard card={c} />
        ))}
      </Box>
    </>
  );
}

function GetGameState({
  gameId,
  step,
  game,
  defenderLog, challengerLog
}: {
  gameId: string;
  step: number;
  game: GameWithBotsWithResult<BotWithTeam<Team>>;
  defenderLog: string | undefined;
  challengerLog: string | undefined;
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
  };

  useEffect(() => fetchData(), [step]);

  return game_state ? (
    <Box
      sx={(theme) => ({
        display: "grid",
        gridTemplateRows: "repeat(3, auto)",
        gridTemplateAreas: '"challenger" "game" "defender"',
        [theme.breakpoints.up("lg")]: {
          gridTemplateColumns: "1fr auto 1fr",
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
        <Box
          sx={{
            flexDirection: "row",
            display: "flex",
            gap: 2,
            alignItems: "center",
          }}
        >
          <Status
            hand={game_state.challenger_hand}
            pushed={game_state.challenger_pushed}
            stack={game_state.challenger_stack}
            position={game_state.sb == "Challenger" ? "SB" : "BB"}
            lastAction={(game_state.sb == 'Challenger') == (game_state.whose_turn == 'SmallBlind') ? game_state.action_val : undefined}
          />
        </Box>
      </Box>
      <Box
        sx={{
          gridArea: "game",
          position: "relative",
        }}
      >
        <Box
          sx={{
            height: "100%",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            gap: 2,
          }}
        >
          <Typography level="h3" color="inherit">
            Pot {game_state.defender_pushed + game_state.challenger_pushed}
          </Typography>
          <Typography level="h3" color="inherit">
            {game_state.end_reason ? endMessage(game_state.end_reason) : (roundName(game_state.community_cards.length))}
          </Typography>
          <GameTable {...game_state} />
        </Box>
      </Box>
      <Box
        sx={{
          display: "flex",
          gridArea: "defender",
          flexDirection: "column",
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
        <Box
          sx={{
            flexDirection: "row-reverse",
            display: "flex",
            gap: 2,
            textAlign: "right",
            alignItems: "center",
          }}
        >
          <Status
            hand={game_state.defender_hand}
            pushed={game_state.defender_pushed}
            stack={game_state.defender_stack}
            position={game_state.sb == "Defender" ? "SB" : "BB"}
            lastAction={(game_state.sb == 'Defender') == (game_state.whose_turn == 'SmallBlind') ? game_state.action_val : undefined}
          />
        </Box>
      </Box>
    </Box>
  ) : (
    <CircularProgress />
  );
}

export default function GameVisualizer({ gameId }: { gameId: string }) {
  const [max, setMax] = useState(0);

  const urlParams = new URLSearchParams(window.location.search);
  const step = parseInt(urlParams.get("step") ?? "0");
  const [inputValue, setInputValue] = useState(step);
  const [game, setGame] = useState<GameWithBotsWithResult<BotWithTeam<Team>>>();

  const [defenderLog, setDefenderLog] = useState<string | null>();
  const [challengerLog, setChallengerLog] = useState<string | null>();

  const [paused, setPaused] = useState(true);

  const handleInputChange = useCallback(
    (event: any) => {
      // update params
      setInputValue(event.target.value);
      urlParams.set("step", event.target.value);

      // update url
      window.history.replaceState(
        {},
        "",
        `${window.location.pathname}?${urlParams.toString()}`
      );
    },
    [setInputValue, inputValue]
  );
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

    fetch(`${apiUrl}/game-log?id=${gameId}`)
      .then((body) => body.text())
      .then((text) => setDefenderLog(text));
  }, [gameId]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (inputValue == max) {
        setPaused(true);
      } else if (!paused) {
        handleInputChange({
          target: { value: Math.min((inputValue ?? -1) + 1, max) },
        });
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [paused, inputValue]);

  if (!game || isNaN(max)) {
    return (
      <HeaderFooter fullWidth graphics={[`url(${bgImage})`]}>
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
    <HeaderFooter fullWidth graphics={[`url(${bgImage})`]}>
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
          color="inherit"
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
        <Box
          sx={{
            mt: 4,
            display: "flex",
            flexDirection: "row",
            alignItems: "center",
            gap: 2,
          }}
        >
          <ButtonGroup>
            <IconButton
              variant="soft"
              onClick={() => {
                handleInputChange({
                  target: { value: Math.max(inputValue - 1, 0) },
                });
              }}
            >
              <SkipPrevious />
            </IconButton>
            {paused ? (
              <IconButton variant="soft" onClick={() => setPaused(false)}>
                <PlayArrow />
              </IconButton>
            ) : (
              <IconButton variant="soft" onClick={() => setPaused(true)}>
                <Pause />
              </IconButton>
            )}
            <IconButton
              variant="soft"
              onClick={() => {
                handleInputChange({
                  target: { value: Math.min(inputValue + 1, max) },
                });
              }}
            >
              <SkipNext />
            </IconButton>
          </ButtonGroup>
          <Typography
            whiteSpace={"nowrap"}
            textAlign={"right"}
            level="h3"
            color="inherit"
          >
            Step {inputValue}/{max}
          </Typography>
          <Slider
            value={inputValue}
            onChange={handleInputChange}
            step={1}
            marks
            min={0}
            max={max}
            variant="solid"
            color="danger"
            sx={(theme) => ({
              width: "100%",
              [theme.breakpoints.down("md")]: {
                display: "none",
              },
            })}
          />
        </Box>
      </Box>
    </HeaderFooter>
  );
}
