import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../state";
import Typography from "@mui/joy/Typography";
import { DataGrid } from "@mui/x-data-grid/DataGrid";
import Chip, { ChipProps } from "@mui/joy/Chip";
import { enqueueSnackbar } from "notistack";
import {
  Box,
  Dropdown,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
} from "@mui/joy";
import { Bot } from "@bindings/Bot";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { BotsResponse } from "@bindings/BotsResponse";
import { BuildStatus } from "@bindings/BuildStatus";
import DataTable, { DataTableColumn } from "../components/DataTable";
import { Check, MoreVert } from "@mui/icons-material";
import BotCard from "./BotCard";

// Group bots by name and output versions
function groupBots<T>(
  bots: BotWithTeam<T>[]
): { bot: BotWithTeam<T>; version: number }[] {
  const grouped: {
    [key: string]: BotWithTeam<T>[];
  } = {};

  for (const bot of bots) {
    if (!(bot.name in grouped)) {
      grouped[bot.name] = [];
    }
    grouped[bot.name].push(bot);
  }

  const output: { bot: BotWithTeam<T>; version: number }[] = [];

  for (const name in grouped) {
    const bots = grouped[name];
    const newest = bots.reduce((a, b) => (a.created > b.created ? a : b));
    output.push({ bot: newest, version: bots.length });
  }

  return output;
}

export default function BotList({
  readonly,
  teamId,
}: {
  readonly?: boolean;
  teamId: string | null;
}) {
  const [team, fetchTeam] = useTeam(teamId ?? null);
  const [bots, setBots] = React.useState<BotWithTeam<Team>[]>([]);
  const [botCount, setBotCount] = React.useState(0);
  const [myTeam, fetchMyTeam] = useTeam(null);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);

  const getBots = () => {
    fetch(`${apiUrl}/bots?team=${team?.id}&count=true`)
      .then((res) => res.json())
      .then((data: BotsResponse) =>
        setBotCount("Count" in data ? Number(data.Count) : 0)
      );

    return fetch(
      `${apiUrl}/bots?join_team=true&page=${paginationModel.page}&page_size=${paginationModel.pageSize}&team=${team?.id}`
    )
      .then((res) => res.json())
      .then(async (data: BotsResponse) => {
        if ("Bots" in data) {
          setLoading(false);
          setBots(data.Bots);
        } else {
          setBots([]);
          enqueueSnackbar("Error loading bots", { variant: "error" });
        }
      });
  };
  useEffect(() => {
    setLoading(true);
    getBots();
    const int = setInterval(() => {
      getBots();
    }, 1000);
    return () => clearInterval(int);
  }, [paginationModel, team?.active_bot]);

  const groups = groupBots(bots);

  return (
    <>
      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
          flexWrap: "wrap",
          gap: 2,
        }}
      >
        {groups.map(({ bot, version }) => {
          return (
            <BotCard
              bot={bot}
              version={version}
              onSetActive={() => handleSetActive(bot.id)}
              onDelete={() =>
                handleDelete(
                  bots.filter((b) => b.name == bot.name).map((b) => b.id)
                )
              }
            ></BotCard>
          );
        })}
      </Box>
    </>
  );

  function handleDelete(botIds: number[]) {
    if (!window.confirm("Are you sure you want to delete a bot?")) return;
    for (const botId of botIds) {
      fetch(`${apiUrl}/delete-bot?id=${botId}`).then(async (res) => {
        if (res.status !== 200) {
          const error = await res.json();
          enqueueSnackbar(`Error deleting bot: ${error.error}`, {
            variant: "error",
          });
        }
        getBots();
        fetchTeam();
      });
    }
  }

  function handleSetActive(botId: number) {
    fetch(`${apiUrl}/set-active-bot?id=${botId}`)
      .then(async (r) => {
        const data = await r.json();
        if (data?.error) {
          enqueueSnackbar(data.error, {
            variant: "error",
          });
        }
      })
      .then(() => {
        enqueueSnackbar("Set active", {
          variant: "success",
        });
        setTimeout(() => {
          fetchTeam();
        }, 100);
      });
  }
}
