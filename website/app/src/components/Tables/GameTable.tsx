import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../../state";
import Box from "@mui/system/Box";
import { styled } from "@mui/joy/styles";
import Button, { ButtonProps } from "@mui/joy/Button";
import {
  Avatar,
  Chip,
  ChipProps,
  Dropdown,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  Typography,
} from "@mui/joy";
import { Link } from "react-router-dom";
import { GridMoreVertIcon } from "@mui/x-data-grid";
import { GameWithBots } from "@bindings/GameWithBots";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { enqueueSnackbar } from "notistack";
import { WhichBot } from "@bindings/WhichBot";
import DataTable, { DataTableProps } from "../DataTable";
import { ArrowRight, MoreVert } from "@mui/icons-material";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";
import GameCard from "../GameCard";

export const TableButton = styled((props: ButtonProps) => (
  <Button
    {...props}
    variant="plain"
    sx={{
      color: "#bbb",
      whiteSpace: 'nowrap',
      background: "none",
      ":hover": {
        background: "#00000040",
      },
      ":active": {
        background: "#00000080",
      },
    }}
    size="sm"
  />
))(() => ({}));

const RatingChange = ({
  before,
  after,
  running,
}: {
  before: number;
  after: number;
  running: boolean;
}) => {
  const change = after - before;
  const color = change > 0 ? "success" : change < 0 ? "danger" : "neutral";
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
      }}
    >
      <Chip color={color} size="sm" key="chip">
        <Typography
          textColor="inherit"
          sx={{
            display: "flex",
            flexDirection: "row",
            alignItems: "center",
            gap: 0.5,
          }}
        >
          {before.toFixed(0)} <ArrowRight /> {after.toFixed(0)}
        </Typography>
      </Chip>
    </Box>
  );
};

type Game = GameWithBotsWithResult<BotWithTeam<Team>>;

const renderScore = ({
  scoreChange,
  errorType,
  whichBot,
  running,
}: {
  scoreChange: number | null;
  errorType: string;
  whichBot: WhichBot;
  running: boolean;
}) => {
  let color: ChipProps["color"] = "success";
  if (running) return <></>;
  if (scoreChange == null) color = "warning";
  else if (scoreChange < 0) color = "danger";
  else if (scoreChange == 0) color = "neutral";
  if (errorType) {
    color = "warning";
  }

  return (
    <Box
      sx={{
        flexDirection: "row",
        justifyContent: whichBot == "Challenger" ? "right" : "left",
        display: "flex",
      }}
    >
      <Chip color={color} size="sm" key="chip">
        {scoreChange === null ? "Running" : errorType ?? scoreChange ?? 0}
      </Chip>
    </Box>
  );
};

export function GameList({ teamId }: { teamId?: string | null }) {
  const [team, fetchTeam] = useTeam(teamId ?? null);
  const [myTeam, fetchMyTeam] = useTeam(null);
  const [games, setGames] = React.useState<Game[]>([]);
  const [gameCount, setGameCount] = React.useState(0);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);

  const getGames = useCallback(() => {
    fetch(
      `${apiUrl}/count-games?${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then((data: number | object) => {
        if (typeof data == "number") setGameCount(data);
      });

    fetch(
      `${apiUrl}/games?page=${paginationModel.page}&page_size=${
        paginationModel.pageSize
      }&${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then(async (data: GameWithBotsWithResult<BotWithTeam<Team>>[]) => {
        // swap teama and teamb if teama is not the user's team
        setLoading(false);
        if (!("error" in data)) {
          setGames(data);
        } else {
          setGames([]);
          enqueueSnackbar("Error loading games", { variant: "error" });
        }
      });
  }, [team?.id, paginationModel.page, paginationModel.pageSize]);
  //TODO: only poll active games
  useEffect(() => {
    setLoading(true);
    getGames();
    const int = setInterval(() => {
      getGames();
    }, 1000);
    return () => clearInterval(int);
  }, [getGames, paginationModel]);

  console.log(games);


  return (
    <>
    {games.map((game) => (
      <GameCard game={game}/>
    ))}
</>
  );
}
