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
      {challenger_hand.map((c) => (
        <GameCard card={c} />
      ))}
    </div>,

    <div className="cards">
      {allCards.map((c) => {
        return <GameCard card={c} />;
      })}
    </div>,
    <div className="cards">
      {defender_hand.map((c) => (
        <GameCard card={c} />
      ))}
    </div>,
  ];
}

function ActionNote({ action }: { action: Action | undefined }) {
  if (!action)
    return (
      <Typography level="h3" color="inherit">
        Did not act
      </Typography>
    );
  else if (action == "Fold") {
    return (
      <Typography level="h3" color="inherit">
        Fold
      </Typography>
    );
  } else if (action.Raise == 0) {
    return (
      <Typography level="h3" color="inherit">
        Check/call
      </Typography>
    );
  } else {
    return (
      <Typography level="h3" color="inherit">
        Raise {action.Raise}
      </Typography>
    );
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

  useEffect(() => fetchData(), [step]);
  console.log("state", game_state);

  const challengerActionNote = (
    <ActionNote
      action={
        (game_state?.whose_turn == "SmallBlind") ==
        (game_state?.sb == "Challenger")
          ? game_state?.action_val
          : undefined
      }
    />
  );

  const defenderActionNote = (
    <ActionNote
      action={
        (game_state?.whose_turn == "SmallBlind") ==
        (game_state?.sb == "Defender")
          ? game_state?.action_val
          : undefined
      }
    />
  );

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
          justifyContent: "stretch",
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
            display: "flex",
            flexDirection: "row",
            justifyContent: "flex-start",
            gap: 2,
          }}
        >
          <KeyValue
            keyName="Pushed"
            value={`${
              game_state.end_reason ? 0 : game_state.challenger_pushed
            }/${game_state.challenger_stack}`}
          />
          <KeyValue
            keyName="Position"
            value={<>{game_state.sb == "Challenger" ? "SB" : "BB"}</>}
          />
        </Box>
        <Box>{challengerActionNote}</Box>
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
            justifyContent: "center",
            alignItems: "center",
            gap: 2,
          }}
        >
          <Typography level="h3" color="inherit">
            Pot {game_state.defender_pushed + game_state.challenger_pushed}
          </Typography>
          <Typography level="h3" color="inherit">
            {game_state.end_reason ? endMessage(game_state.end_reason) : ""}
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
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            justifyContent: "flex-end",
            gap: 2,
          }}
        >
          <KeyValue
            keyName="Pushed"
            value={
              <>
                {game_state.end_reason ? 0 : game_state.defender_pushed}/
                {game_state.defender_stack}
              </>
            }
          />
          <KeyValue
            keyName="Position"
            value={<>{game_state.sb == "Defender" ? "SB" : "BB"}</>}
          />
        </Box>
        <Box textAlign="right">{defenderActionNote}</Box>
      </Box>
    </Box>
  ) : (
    <CircularProgress />
  );
}

export default function GameVisualizer({ gameId }: { gameId: string }) {
  const [max, setMax] = useState(0);

  const urlParams = new URLSearchParams(window.location.search);
  const step = urlParams.get("step");
  const [inputValue, setInputValue] = useState(step ?? 0);
  const [game, setGame] = useState<GameWithBotsWithResult<BotWithTeam<Team>>>();

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
  }, [gameId]);

  useEffect(() => {
    const interval = setInterval(() => {
      if (!paused) {
        handleInputChange({ target: { value: inputValue + 1 } });
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
