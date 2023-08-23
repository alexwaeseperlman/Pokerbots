import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../state";
import Typography from "@mui/joy/Typography";
import { DataGrid } from "@mui/x-data-grid/DataGrid";
import Chip, { ChipProps } from "@mui/joy/Chip";
import { enqueueSnackbar } from "notistack";
import { Dropdown, IconButton, Menu, MenuButton, MenuItem } from "@mui/joy";
import { Bot } from "@bindings/Bot";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { BotsResponse } from "@bindings/BotsResponse";
import { BuildStatus } from "@bindings/BuildStatus";
import DataTable from "../components/DataTable";
import { MoreVert } from "@mui/icons-material";

export default function BotTable({
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
        if ("BotsWithTeam" in data) {
          setLoading(false);
          setBots(data.BotsWithTeam);
        } else {
          setBots([]);
          enqueueSnackbar("Error loading bots", { variant: "error" });
          console.error("Received bots as", data);
        }
      });
  };
  useEffect(() => {
    setLoading(true);
    getBots();
    const int = setInterval(() => {
      getBots();
    }, 5000);
    return () => clearInterval(int);
  }, [paginationModel, team?.active_bot]);

  return (
    <>
      <DataTable
        columns={[
          {
            name: "Result",
            render: ({ row: bot }) => {
              let [color, message] = (
                {
                  Unqueued: ["warning", "Not in queue"],
                  Queued: ["neutral", "In queue"],
                  Building: ["neutral", "Building"],
                  BuildSucceeded: ["neutral", "Built successfully"],
                  PlayingTestGame: ["neutral", "Playing test game"],
                  TestGameSucceeded: ["success", "Ready to play"],
                  BuildFailed: ["danger", "Build failed"],
                  TestGameFailed: ["danger", "Test game failed"],
                } as Record<BuildStatus, [ChipProps["color"], string]>
              )[bot.build_status];
              return <Chip color={color}>{message}</Chip>;
            },
          },
          {
            name: "Uploaded By",
            render: ({ row: bot }) => (
              <Typography level="body-md">{bot.uploaded_by}</Typography>
            ),
          },
          {
            name: "Name",
            render: ({ row: bot }) => (
              <Typography level="body-md">{bot.name}</Typography>
            ),
          },
          {
            name: "Uploaded",
            render: ({ row: bot }) => {
              const date = new Date(Number(bot.created) * 1000);
              return (
                <Typography>
                  {date.toLocaleDateString()} {date.toLocaleTimeString()}
                </Typography>
              );
            },
          },
          {
            name: "",
            width: 40,
            render: ({ row: bot }) => {
              const ref = React.useRef(null);
              return (
                <Dropdown>
                  <MenuButton
                    slots={{ root: IconButton }}
                    slotProps={{
                      root: { variant: "outlined", color: "neutral" },
                    }}
                  >
                    <MoreVert />
                  </MenuButton>

                  <Menu>
                    <MenuItem onClick={handleSetActive(bot)}>
                      {bot.id == team?.active_bot
                        ? "Currently active"
                        : "Set active"}
                    </MenuItem>

                    <MenuItem onClick={handleChallenge(bot)}>
                      Challenge
                    </MenuItem>

                    <MenuItem
                      target="_tab"
                      href={`${apiUrl}/build-log?bot=${bot.id}`}
                      component="a"
                    >
                      Get build log
                    </MenuItem>
                    <MenuItem onClick={handleDelete(bot)} color="danger">
                      Delete
                    </MenuItem>
                  </Menu>
                </Dropdown>
              );
            },
          },
        ]}
        loading={loading}
        data={bots}
        perPage={paginationModel.pageSize}
        onPageChange={(page) => {
          setPaginationModel({
            ...paginationModel,
            page,
          });
        }}
        serverPagination={true}
        total={botCount ?? 0}
      />
    </>
  );

  function handleDelete(
    bot: BotWithTeam<Team>
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
      if (!window.confirm("Are you sure you want to delete a bot?")) return;
      fetch(`${apiUrl}/delete-bot?id=${bot.id}`).then(() => getBots());
    };
  }

  function handleChallenge(
    bot: BotWithTeam<Team>
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
      fetch(
        `${apiUrl}/create-game?challenger=${myTeam?.active_bot}&defender=${bot.id}`
      ).then(async (r) => {
        const data = await r.json();
        if (data.error) {
          enqueueSnackbar(data.error, { variant: "error" });
        }
      });
    };
  }

  function handleSetActive(
    bot: BotWithTeam<Team>
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
      fetch(`${apiUrl}/set-active-bot?id=${bot.id}`)
        .then(async (r) => {
          const data = await r.json();
          if (data.error) {
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
    };
  }
}
