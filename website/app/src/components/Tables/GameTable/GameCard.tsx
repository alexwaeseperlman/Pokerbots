import { BotWithTeam } from "@bindings/BotWithTeam";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";
import { Team } from "@bindings/Team";
import { WhichBot } from "@bindings/WhichBot";
import {
  Avatar,
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  CircularProgress,
  IconButton,
  Stack,
  Typography,
} from "@mui/joy";
import * as React from "react";
import { apiUrl } from "../../../state";
import { Link } from "react-router-dom";
import { GameError } from "@bindings/GameError";
import { ErrorOutline, PlayArrow, Replay } from "@mui/icons-material";

function RatingChange({
  ratingChange,
  rating,
}: {
  ratingChange: number | undefined;
  rating: number | undefined;
}) {
  if (ratingChange === undefined || rating === undefined) {
    return <></>;
  }
  const color =
    ratingChange > 0 ? "success" : ratingChange < 0 ? "danger" : "neutral";
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        justifyContent: "center",
        ml: 2,
        mr: 2,
      }}
    >
      <Typography
        fontSize="small"
        sx={{
          opacity: 0.5,
        }}
        textColor="text.secondary"
      >
        {rating.toFixed(0)}
      </Typography>
      &nbsp;
      <Typography color={color}>
        {ratingChange > 0 ? "+" : ""}
        {ratingChange.toFixed(0)}
      </Typography>
    </Box>
  );
}

function TeamObject({
  botName,
  direction,
  teamId,
  teamName,
  ratingChange,
  rating,
  error,
}: {
  botName: string;
  direction: WhichBot;
  teamId: number;
  teamName: string;
  ratingChange: number | undefined;
  rating: number | undefined;
  error: GameError | null | undefined;
}) {
  let error_message = null;
  if (error) {
    if (error == "InternalError") {
      error_message = "Internal error";
    } else if ("RunTimeError" in error && error.RunTimeError == direction) {
      error_message = "Runtime error";
    } else if ("TimeoutError" in error && error.TimeoutError == direction) {
      error_message = "Timeout error";
    } else if ("MemoryError" in error && error.MemoryError == direction) {
      error_message = "Runtime error";
    } else if (
      "InvalidActionError" in error &&
      error.InvalidActionError == direction
    ) {
      error_message = "Invalid action error";
    }
  }

  return (
    <Stack
      sx={{
        flexDirection: "column",
        alignItems: "stretch",
        flex: "1 0 45%",
      }}
    >
      <Box
        sx={{
          flexDirection: direction == "Challenger" ? "row" : "row-reverse",
          display: "flex",
          justifyContent: "space-between",
        }}
      >
        <Typography
          level="title-lg"
          textAlign={direction == "Challenger" ? "left" : "right"}
          whiteSpace={"nowrap"}
        >
          {direction}
        </Typography>
        <RatingChange rating={rating} ratingChange={ratingChange} />
      </Box>
      <Box
        sx={{
          display: "flex",
          flexDirection: direction == "Challenger" ? "row" : "row-reverse",
          alignItems: "center",
          justifyContent: direction,
        }}
      >
        <Box
          key="team"
          sx={{
            display: "flex",
            flexDirection: direction == "Challenger" ? "row" : "row-reverse",
            alignItems: "center",
          }}
        >
          <Avatar
            key="avatar"
            sx={{
              width: 32,
              height: 32,
            }}
            src={`${apiUrl}/pfp?id=${teamId}`}
          />
          <Box
            key="name"
            ml={2}
            mr={2}
            flexDirection={"column"}
            textAlign={direction == "Challenger" ? "left" : "right"}
          >
            <Link
              to={`/team/${teamId}`}
              style={{
                color: "inherit",
                textDecoration: "none",
              }}
            >
              <Typography whiteSpace={"nowrap"}>
                {teamName ?? "Deleted team"}
              </Typography>
            </Link>

            <Typography
              fontSize="small"
              textColor="text.secondary"
              whiteSpace="nowrap"
            >
              {botName ?? "Deleted bot"}
            </Typography>
          </Box>
        </Box>
      </Box>
      {error_message && (
        <Typography
          textColor={"warning.500"}
          display={"flex"}
          alignItems={"center"}
          justifyContent={direction == "Challenger" ? "left" : "right"}
          gap={1}
        >
          <ErrorOutline color="warning" />
          {error_message}
        </Typography>
      )}
    </Stack>
  );
}

export default function GameCard(props: {
  game: GameWithBotsWithResult<BotWithTeam<Team>>;
}) {
  return (
    <Card size="md">
      <CardContent
        sx={{
          flexDirection: "row",
          justifyContent: "space-between",
        }}
      >
        <Box
          sx={{
            flexDirection: "row-reverse",
            display: "flex",
            flexWrap: "wrap",
            flexGrow: 1,
            maxWidth: "700px",
            gap: 4,
          }}
        >
          <TeamObject
            botName={props.game.challenger.name}
            direction="Challenger"
            teamId={props.game.challenger.team.id}
            teamName={props.game.challenger.team.name}
            rating={props.game.result?.challenger_rating}
            ratingChange={props.game.result?.challenger_rating_change}
            error={props.game.result?.error_type}
          />
          <TeamObject
            botName={props.game.defender.name}
            direction="Defender"
            teamId={props.game.defender.team.id}
            teamName={props.game.defender.team.name}
            rating={props.game.result?.defender_rating}
            ratingChange={props.game.result?.defender_rating_change}
            error={props.game.result?.error_type}
          />
        </Box>
        <Button
          sx={{
            height: "100%",
            whiteSpace: "nowrap",
          }}
          variant="plain"
          color="neutral"
        >
          Replay
        </Button>
      </CardContent>
    </Card>
  );
}
