import React, { useCallback, useEffect } from "react";
import { Bot, Team, apiUrl, pfpEndpoint, useMyTeam, useTeam } from "../state";
import Avatar from "@mui/material/Avatar";
import Box from "@mui/system/Box";
import Typography from "@mui/material/Typography";
import { DataGrid } from "@mui/x-data-grid/DataGrid";
import Chip from "@mui/material/Chip";
import { TableButton } from ".";
import Button from "@mui/material/Button";
import { enqueueSnackbar } from "notistack";

export default function BotTable({ readonly }: { readonly?: boolean }) {
  const [team, fetchTeam] = useTeam();
  const [bots, setBots] = React.useState<(Bot & { active: boolean })[]>([]);
  const [botCount, setBotCount] = React.useState(0);
  const [myTeam, fetchMyTeam] = useMyTeam();
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);
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
              return <Chip color="default" label={"Built successfully"}></Chip>;
            if (params.row.build_status === 3)
              return <Chip color="default" label={"Playing test game"}></Chip>;
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
        ...(readonly
          ? []
          : [
              {
                field: "delete-col",
                headerName: "",
                minWidth: 150,
                sortable: false,
                valueGetter(params) {
                  return params.id;
                },
                renderCell: (params) => {
                  // delete that bot
                  return (
                    <Button
                      sx={{
                        color: "black",
                      }}
                      onClick={() => {
                        if (
                          !window.confirm(
                            "Are you sure you want to delete a bot?"
                          )
                        )
                          return;
                        fetch(`${apiUrl}/delete-bot?id=${params.value}`).then(
                          () => getBots()
                        );
                      }}
                    >
                      Delete
                    </Button>
                  );
                },
              },
              {
                field: "id",
                headerName: "",
                minWidth: 175,
                sortable: false,
                valueGetter: (params) => ({
                  active: params.row.active,
                  id: params.value,
                }),
                renderCell: (params) => {
                  // make that bot active
                  return (
                    <Button
                      sx={{
                        color: "black",
                      }}
                      onClick={() => {
                        fetch(`${apiUrl}/set-active-bot?id=${params.id}`)
                          .then(async (r) => {
                            const data = await r.json();
                            if (data.error) {
                              enqueueSnackbar(data.error, { variant: "error" });
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
                      {params.value.active ? "Currently active" : "Set active"}
                    </Button>
                  );
                },
              },
            ]),
        {
          field: "play-against",
          headerName: "",
          minWidth: 150,
          sortable: false,
          renderCell: (params) => {
            return (
              <Button
                sx={{
                  color: "black",
                }}
                onClick={() => {
                  fetch(
                    `${apiUrl}/make-game?bot_a=${myTeam?.active_bot}&bot_b=${params.id}`
                  ).then(async (r) => {
                    const data = await r.json();
                    if (data.error) {
                      enqueueSnackbar(data.error, { variant: "error" });
                    }
                  });
                }}
              >
                Challenge
              </Button>
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
      rowCount={botCount}
      onPaginationModelChange={setPaginationModel}
      disableColumnFilter
      disableColumnMenu
      disableColumnSelector
      disableDensitySelector
      disableRowSelectionOnClick
    />
  );
}
