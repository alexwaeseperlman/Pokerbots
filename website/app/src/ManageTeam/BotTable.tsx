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
import DataTable, { DataTableColumn } from "../components/DataTable";
import { Check, MoreVert } from "@mui/icons-material";

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

  const columns: DataTableColumn<BotWithTeam<Team>>[] = React.useMemo(
    () => [
      {
        name: "Active",
        key: "active",
        width: "75px",
        getProps: (bot) => ({ id: bot.id }),
        render: ({ id }) => {
          if (id == team?.active_bot)
            return (
              <Check display={id == team?.active_bot ? "block" : "none"} />
            );
          return <></>;
        },
      },
      {
        name: "Result",
        key: "result",
        getProps: (bot) => ({ buildStatus: bot.build_status }),
        render: ({ buildStatus }) => {
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
          )[buildStatus as BuildStatus];
          return <Chip color={color}>{message}</Chip>;
        },
      },
      {
        name: "Name",
        key: "name",
        getProps: (bot) => ({ name: bot.name }),
        render: ({ name }) => (
          <Typography
            level="body-md"
            textOverflow={"ellipsis"}
            overflow={"inherit"}
          >
            {name}
          </Typography>
        ),
      },
      {
        name: "Uploaded By",
        key: "uploaded by",
        minWidth: "200px",
        maxWidth: "500px",
        getProps: (bot) => ({ uploadedBy: bot.uploaded_by.display_name }),
        render: ({ uploadedBy }) => (
          <Typography
            level="body-md"
            textOverflow={"ellipsis"}
            overflow={"inherit"}
          >
            {uploadedBy}
          </Typography>
        ),
      },
      {
        name: "Uploaded",
        key: "uploaded",
        getProps: (bot) => ({ created: bot.created.toString() }),
        render: ({ created }) => {
          const date = new Date(Number(created) * 1000);
          return (
            <Typography
              level="body-md"
              textOverflow={"ellipsis"}
              overflow={"inherit"}
            >
              {date.toLocaleDateString()} {date.toLocaleTimeString()}
            </Typography>
          );
        },
      },
      {
        name: "",
        key: "actions",
        width: 40,
        getProps: (bot) => ({ id: bot.id }),
        render: ({ id }: { id: number }) => {
          const ref = React.useRef(null);

          return (
            <Dropdown>
              <MenuButton
                slots={{ root: IconButton }}
                slotProps={{
                  root: { variant: "plain", color: "neutral" },
                }}
              >
                <MoreVert />
              </MenuButton>

              <Menu>
                <MenuItem onClick={handleSetActive(id)}>
                  {id == team?.active_bot ? "Currently active" : "Set active"}
                </MenuItem>

                {/*<MenuItem onClick={handleChallenge(id)}>Challenge</MenuItem>*/}

                <MenuItem
                  target="_tab"
                  href={`${apiUrl}/build-log?bot=${id}`}
                  component="a"
                >
                  Get build log
                </MenuItem>
                <MenuItem onClick={handleDelete(id)} color="danger">
                  Delete
                </MenuItem>
              </Menu>
            </Dropdown>
          );
        },
      },
    ],
    [team?.active_bot, team?.id]
  );

  return (
    <>
      <DataTable<BotWithTeam<Team>>
        columns={columns}
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
    botId: number
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
      if (!window.confirm("Are you sure you want to delete a bot?")) return;
      fetch(`${apiUrl}/delete-bot?id=${botId}`).then(() => getBots());
    };
  }

  function handleChallenge(
    botId: number
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
      fetch(
        `${apiUrl}/create-game?challenger=${myTeam?.active_bot}&defender=${botId}`
      ).then(async (r) => {
        const data = await r.json();
        if (data.error) {
          enqueueSnackbar(data.error, { variant: "error" });
        }
      });
    };
  }

  function handleSetActive(
    botId: number
  ): React.MouseEventHandler<HTMLDivElement> | undefined {
    return () => {
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
    };
  }
}
