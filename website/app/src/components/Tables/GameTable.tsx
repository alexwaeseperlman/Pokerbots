import React, { useCallback, useEffect } from "react";
import {
  Game,
  apiUrl,
  usePfpEndpoint,
  fillInGames,
  useTeam,
} from "../../state";
import Box from "@mui/system/Box";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button, { ButtonProps } from "@mui/material/Button";
import { Avatar, Chip, ChipProps, Typography } from "@mui/material";
import { Link } from "react-router-dom";

export const DataGrid = React.lazy(() =>
  import("@mui/x-data-grid").then((mod) => ({ default: mod.DataGrid }))
);

export const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
export const TableButton = styled((props: ButtonProps) => (
  <Button {...props} />
))(() => ({
  fontSize: "12px",
  fontWeight: 300,
  textAlign: "left",
  justifyContent: "left",
  textTransform: "none",
  cursor: "pointer",
  padding: 0,
  paddingLeft: "8px",
  paddingRight: "8px",
  color: "#bbb",
}));
export function GameTable({ teamId }: { teamId?: string | null }) {
  const [team, fetchTeam] = useTeam(teamId ?? null);
  const [games, setGames] = React.useState<Game[]>([]);
  const [gameCount, setGameCount] = React.useState(0);
  const [pfpEndpoint, fetchPfpEndpoint] = usePfpEndpoint();
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);
  const getGames = useCallback(() => {
    fetch(
      `${apiUrl}/games?${
        teamId === undefined ? "" : `team=${team?.id}`
      }&count=true`
    )
      .then((res) => res.json())
      .then((data) => setGameCount(data.count));

    fetch(
      `${apiUrl}/games?page=${paginationModel.page}&page_size=${
        paginationModel.pageSize
      }&${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then(async (data) => {
        // swap teama and teamb if teama is not the user's team
        const games = data;
        setLoading(false);
        setGames(await fillInGames(games));
      });
  }, [team?.id, paginationModel.page, paginationModel.pageSize]);
  //TODO: only poll active games
  useEffect(() => {
    setLoading(true);
    getGames();
    const int = setInterval(() => {
      getGames();
    }, 5000);
    return () => clearInterval(int);
  }, [getGames, paginationModel]);
  const renderTeam = (params) => (
    <>
      <Avatar
        sx={{
          width: 24,
          height: 24,
          marginRight: 2,
        }}
        src={`${pfpEndpoint}${params.value?.team?.id}`}
      />
      <Box flexDirection={"column"}>
        <Link
          to={`/team/${params.value?.team?.id}`}
          style={{
            color: "inherit",
            textDecoration: "none",
          }}
        >
          <Typography>
            {params.value?.team?.team_name ?? "Deleted team"}
          </Typography>
        </Link>

        <Typography fontSize="small" color={"text.secondary"}>
          {params.value?.name ?? "Deleted bot"}
        </Typography>
      </Box>
    </>
  );

  return (
    <DataGrid
      columns={[
        {
          field: "score_change",
          headerName: "Result",
          renderCell: (params) => {
            if (params.value === null)
              return <Chip color="default" label={"Running"}></Chip>;
            let color: ChipProps["color"] = "success";
            if (params.value < 0) color = "error";
            else if (params.value == 0) color = "default";
            console.log(params.row);
            if (params.row.error_type) {
              color = "warning";
            }
            return (
              <Chip
                label={params.row.error_type ?? params.value}
                color={color}
              />
            );
          },
          flex: 1,
          sortable: false,
        },
        {
          field: "bot_a",
          headerName: "Defender",
          renderCell: renderTeam,
          flex: 1,
          sortable: false,
        },
        {
          field: "bot_b",
          headerName: "Challenger",
          renderCell: renderTeam,
          flex: 1,
          sortable: false,
        },
        {
          field: "game-log",
          headerName: "",
          minWidth: 150,
          sortable: false,
          renderCell: (params) => {
            return (
              <Button
                sx={{
                  color: "black",
                }}
                href={`${apiUrl}/game-log?id=${params.id}`}
              >
                Game log
              </Button>
            );
          },
        },
      ]}
      loading={loading}
      rows={games}
      pagination
      pageSizeOptions={[10, 25, 50, 100]}
      paginationMode="server"
      paginationModel={paginationModel}
      rowCount={gameCount}
      onPaginationModelChange={setPaginationModel}
      disableColumnFilter
      disableColumnMenu
      disableColumnSelector
      disableDensitySelector
      disableRowSelectionOnClick
      sx={{
        width: "100%",
        height: "100%",
      }}
    />
  );
}
