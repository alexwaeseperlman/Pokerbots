import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { WhichBot } from "@bindings/WhichBot";
import { Avatar, Box, Typography } from "@mui/joy";
import * as React from "react";
import { apiUrl } from "../../../state";
import { Link } from "react-router-dom";

export function TeamBotStack({
  bot,
  direction,
  size,
}: {
  bot: BotWithTeam<Team>;
  direction: WhichBot;
  size?: "large" | "small";
}) {
  return (
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
            width: size == "large" ? 64 : 32,
            height: size == "large" ? 64 : 32,
          }}
          src={`${apiUrl}/pfp?id=${bot.team.id}`}
        />
        <Box
          key="name"
          ml={2}
          mr={2}
          flexDirection={"column"}
          textAlign={direction == "Challenger" ? "left" : "right"}
        >
          <Link
            to={`/team/${bot.team.id}`}
            style={{
              color: "inherit",
              textDecoration: "none",
            }}
          >
            <Typography
              color="inherit"
              whiteSpace={"nowrap"}
              level={size == "large" ? "h2" : undefined}
            >
              {bot.team.name ?? "Deleted team"}
            </Typography>
          </Link>

          <Typography
            fontSize={'small'}
            level={size == "large" ? "h3" : undefined}
            color="inherit"
            whiteSpace="nowrap"
          >
            {bot.name ?? "Deleted bot"}
          </Typography>
        </Box>
      </Box>
    </Box>
  );
}
