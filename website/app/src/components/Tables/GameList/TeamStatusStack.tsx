import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { WhichBot } from "@bindings/WhichBot";
import { Box, Stack, Typography } from "@mui/joy";
import * as React from "react";
import { GameError } from "@bindings/GameError";
import { ErrorOutline } from "@mui/icons-material";
import { TeamBotStack } from "./BotCard";
import { RatingChange } from "./GameCard";

export function TeamStatusStack({
  direction,
  ratingChange,
  rating,
  error,
  bot,
  size,
}: {
  direction: WhichBot;
  ratingChange: number | undefined;
  rating: number | undefined;
  error: GameError | null | undefined;
  bot: BotWithTeam<Team>;
  size?: "small" | "large";
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
        flex: "1 0 45%",
      }}
    >
      <Box
        sx={{
          flexDirection: direction == "Challenger" ? "row" : "row-reverse",
          display: "flex",
          //justifyContent: "space-between",
        }}
      >
        <Typography
          level="title-lg"
          textAlign={direction == "Challenger" ? "left" : "right"}
          whiteSpace={"nowrap"}
          color="inherit"
        >
          {direction}
        </Typography>
        <RatingChange rating={rating} ratingChange={ratingChange} />
      </Box>
      <TeamBotStack direction={direction} bot={bot} size={size} />
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
