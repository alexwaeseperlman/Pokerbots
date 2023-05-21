import React, { useCallback, useEffect } from "react";
import { Bot, apiUrl, pfpEndpoint, useTeam } from "../state";
import Avatar from "@mui/material/Avatar";
import Box from "@mui/system/Box";
import Typography from "@mui/material/Typography";
import { DataGrid } from "@mui/x-data-grid/DataGrid";
import Chip from "@mui/material/Chip";
import { TableButton } from ".";
import Button from "@mui/material/Button";

export default function BotTable(props: { readonly?: boolean }) {
  const team = useTeam()[0];
  const [bots, setBots] = React.useState<Bot[]>([]);
  const [botCount, setBotCount] = React.useState(0);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);
  const getBots = useCallback(() => {
    fetch(`${apiUrl}/bots?team=${team?.id}&count=true`)
      .then((res) => res.json())
      .then((data) => setBotCount(data.count));

    fetch(
      `${apiUrl}/bots?page=${paginationModel.page}&page_size=${paginationModel.pageSize}&team=${team?.id}`
    )
      .then((res) => res.json())
      .then(async (data) => {
        // swap teama and teamb if teama is not the user's team
        setLoading(false);
        setBots(data);
      });
  }, [team?.id, paginationModel.page, paginationModel.pageSize]);
  //TODO: only poll active games
  useEffect(() => {
    setLoading(true);
    getBots();
    const int = setInterval(() => {
      getBots();
    }, 5000);
    return () => clearInterval(int);
  }, [getBots, paginationModel]);
  const renderTeam = (params) => (
    <>
      <Avatar
        sx={{
          width: 24,
          height: 24,
          marginRight: 2,
        }}
        src={`${pfpEndpoint}${params.value?.team.id}`}
      />
      <Box flexDirection={"column"}>
        <Typography>{params.value?.team.team_name}</Typography>

        <Typography color={"text.secondary"}>{params.value?.name}</Typography>
      </Box>
    </>
  );

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
        ...(props.readonly
          ? []
          : [
              {
                field: "id",
                headerName: "",
                minWidth: 150,
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
            ]),
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
