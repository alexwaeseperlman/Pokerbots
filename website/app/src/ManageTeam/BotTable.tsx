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
            if (params.value === null)
              return <Chip color="default" label={"Running"}></Chip>;
            let color = "success";
            if (params.value < 0) color = "error";
            else if (params.value == 0) color = "default";
            return <Chip label={params.value} color={color} />;
          },
          minWidth: 100,
          flex: 1,
        },
        {
          field: "uploaded_by",
          headerName: "Uploaded By",
          minWidth: 200,
          flex: 1,
        },
        {
          field: "name",
          headerName: "Name",
          minWidth: 150,
          flex: 1,
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
        },
        ...(readonly
          ? []
          : [
              {
                field: "delete-col",
                headerName: "",
                minWidth: 150,
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
                            fetchTeam();
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
          renderCell: (params) => {
            return (
              <Button
                sx={{
                  color: "black",
                }}
                onClick={() => {
                  fetch(
                    `${apiUrl}/make-game?bot_a=${team?.active_bot}&bot_b=${params.id}`
                  ).then(async (r) => {
                    const data = await r.json();
                    if (data.error) {
                      enqueueSnackbar(data.error, { variant: "error" });
                    }
                  });
                }}
              >
                Play against
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
    />
  );
}
