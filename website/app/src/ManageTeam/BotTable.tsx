import React, { useCallback, useEffect } from "react";
import { Bot, Team, apiUrl, usePfpEndpoint, useTeam } from "../state";
import Typography from "@mui/material/Typography";
import { DataGrid } from "@mui/x-data-grid/DataGrid";
import Chip from "@mui/material/Chip";
import { enqueueSnackbar } from "notistack";
import { IconButton, Menu, MenuItem } from "@mui/material";
import { GridMoreVertIcon } from "@mui/x-data-grid";

export default function BotTable({
  readonly,
  teamId,
}: {
  readonly?: boolean;
  teamId: string | null;
}) {
  const [team, fetchTeam] = useTeam(teamId ?? null);
  const [bots, setBots] = React.useState<(Bot & { active: boolean })[]>([]);
  const [botCount, setBotCount] = React.useState(0);
  const [myTeam, fetchMyTeam] = useTeam(null);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [pfpEndpoint, fetchPfpEndpoint] = usePfpEndpoint();
  const [loading, setLoading] = React.useState(true);

  const [menuEl, setMenuEl] = React.useState<null | {
    bot: Bot;
    el: HTMLElement;
  }>(null);
  const [menuOpen, setMenuOpen] = React.useState(false);

  const getBots = () => {
    fetch(`${apiUrl}/bots?team=${team?.id}&count=true`)
      .then((res) => res.json())
      .then((data) => setBotCount(data.count));

    return fetch(
      `${apiUrl}/bots?page=${paginationModel.page}&page_size=${paginationModel.pageSize}&team=${team?.id}`
    )
      .then((res) => res.json())
      .then(async (data) => {
        // swap teama and teamb if teama is not the user's team
        setLoading(false);
        setBots(
          data.map((bot: Bot) => ({
            ...bot,
            active: bot.id == team?.active_bot,
          }))
        );
      });
  };
  //TODO: only poll active games
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
      <DataGrid
        columns={[
          {
            field: "score",
            headerName: "Result",
            renderCell: (params) => {
              if (params.row.build_status === -1)
                return <Chip color="warning" label={"Not in queue"}></Chip>;
              if (params.row.build_status === 0)
                return <Chip color="default" label={"In queue"}></Chip>;
              if (params.row.build_status === 1)
                return <Chip color="default" label={"Building"}></Chip>;
              if (params.row.build_status === 2)
                return (
                  <Chip color="default" label={"Built successfully"}></Chip>
                );
              if (params.row.build_status === 3)
                return (
                  <Chip color="default" label={"Playing test game"}></Chip>
                );
              // 4 means the bot succeeded in the test game, so we show its score
              if (params.row.build_status === 4)
                return <Chip label={"Ready to play"} color={"success"} />;
              if (params.row.build_status === 5)
                return <Chip color="error" label={"Build failed"}></Chip>;
              if (params.row.build_status == 6)
                return <Chip color="error" label={"Test game failed"}></Chip>;
            },
            minWidth: 100,
            flex: 1,
            sortable: false,
          },
          {
            field: "uploaded_by",
            headerName: "Uploaded By",
            minWidth: 200,
            flex: 1,
            sortable: false,
          },
          {
            field: "name",
            headerName: "Name",
            minWidth: 150,
            flex: 1,
            sortable: false,
          },
          {
            field: "created",
            headerName: "Uploaded",
            minWidth: 150,
            flex: 1,
            renderCell: (params) => {
              const date = new Date(params.value * 1000);
              return (
                <Typography>
                  {date.toLocaleDateString()} {date.toLocaleTimeString()}
                </Typography>
              );
            },
            sortable: false,
          },
          {
            field: "options",
            headerName: "",
            width: 40,
            sortable: false,
            align: "center",
            renderCell: (params) => {
              const ref = React.useRef(null);
              return (
                <IconButton
                  sx={{
                    color: "black",
                  }}
                  ref={ref}
                  onClick={() => {
                    if (menuEl == ref.current) {
                      setMenuEl(null);
                      setMenuOpen(false);
                    } else {
                      setMenuOpen(true);
                      setMenuEl({
                        bot: params.row,
                        el: ref.current as unknown as HTMLElement,
                      });
                    }
                  }}
                >
                  <GridMoreVertIcon />
                </IconButton>
              );
            },
          },
        ]}
        loading={loading}
        rows={bots}
        pagination
        pageSizeOptions={[10, 25, 50, 100]}
        paginationMode="server"
        paginationModel={paginationModel}
        rowCount={botCount ?? 0}
        onPaginationModelChange={setPaginationModel}
        disableColumnFilter
        disableColumnMenu
        disableColumnSelector
        disableDensitySelector
        disableRowSelectionOnClick
      />
      <Menu
        id="bots-menu"
        open={!!menuEl && menuOpen}
        anchorEl={menuEl?.el}
        onClose={() => setMenuOpen(false)}
        onClick={() => setMenuOpen(false)}
      >
        <MenuItem
          onClick={() => {
            fetch(`${apiUrl}/set-active-bot?id=${menuEl?.bot.id}`)
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
          }}
        >
          {menuEl?.bot.active ? "Currently active" : "Set active"}
        </MenuItem>

        <MenuItem
          component="a"
          onClick={() => {
            fetch(
              `${apiUrl}/create-game?defender=${myTeam?.active_bot}&challenger=${menuEl?.bot.id}`
            ).then(async (r) => {
              const data = await r.json();
              if (data.error) {
                enqueueSnackbar(data.error, { variant: "error" });
              }
            });
          }}
        >
          Challenge
        </MenuItem>

        <MenuItem
          target="_tab"
          href={`${apiUrl}/build-log?bot=${menuEl?.bot.id}`}
          component="a"
        >
          Get build log
        </MenuItem>
        <MenuItem
          onClick={() => {
            if (!window.confirm("Are you sure you want to delete a bot?"))
              return;
            fetch(`${apiUrl}/delete-bot?id=${menuEl?.bot.id}`).then(() =>
              getBots()
            );
          }}
          sx={{
            color: "red",
          }}
          component="a"
        >
          Delete
        </MenuItem>
      </Menu>
    </>
  );
}
