import { BotWithTeam } from "@bindings/BotWithTeam";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";
import { Team } from "@bindings/Team";
import {
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  CircularProgress,
  IconButton,
  Typography,
} from "@mui/joy";
import * as React from "react";
import { useNavigate } from "react-router-dom";
import { PlayArrow, Replay } from "@mui/icons-material";
import { TeamStatusStack } from "./TeamStatusStack";

export function RatingChange({
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
        color='inherit'
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

export default function GameCard(props: {
  game: GameWithBotsWithResult<BotWithTeam<Team>>;
}) {
  const navigate = useNavigate();
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
          <TeamStatusStack
            bot={props.game.challenger}
            direction="Challenger"
            rating={props.game.result?.challenger_rating}
            ratingChange={props.game.result?.challenger_rating_change}
            error={props.game.result?.error_type}
          />
          <TeamStatusStack
            bot={props.game.defender}
            direction="Defender"
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
          onClick={() => {
            navigate(`/view-game/${props.game.id}`);
          }}
        >
          Replay
        </Button>
      </CardContent>
    </Card>
  );
}
