import React, { useCallback, useEffect } from "react";
import {
  Game,
  apiUrl,
  useMyTeam,
  useUser,
  Team,
  pfpEndpoint,
  fillInGames,
  useTeam,
} from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button, { ButtonProps } from "@mui/material/Button";

import { secondary_background } from "../styles.module.css";
import { TeamBar } from "./TeamBar";
import { Avatar, Chip, ChipProps, Typography } from "@mui/material";
import BotTable from "./BotTable";
import { BotUpload } from "./BotUpload";

const DataGrid = React.lazy(() =>
  import("@mui/x-data-grid").then((mod) => ({ default: mod.DataGrid }))
);

export const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
export const TableButton = styled((props: ButtonProps) => (
  <Button {...props} color="secondary" />
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
}));

function GameTable({ readonly }: { readonly?: boolean }) {
  const [team, fetchTeam] = useTeam();
  const [games, setGames] = React.useState<Game[]>([]);
  const [gameCount, setGameCount] = React.useState(0);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);
  const getGames = useCallback(() => {
    fetch(`${apiUrl}/games?team=${team?.id}&count=true`)
      .then((res) => res.json())
      .then((data) => setGameCount(data.count));

    fetch(
      `${apiUrl}/games?page=${paginationModel.page}&page_size=${paginationModel.pageSize}&team=${team?.id}`
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
        src={`${pfpEndpoint}${params.value?.team.id}`}
      />
      <Box flexDirection={"column"}>
        <Typography>{params.value?.team.team_name}</Typography>

        <Typography fontSize="small" color={"text.secondary"}>
          {params.value?.name}
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
          minWidth: 100,
          flex: 1,
          sortable: false,
        },
        {
          field: "bot_a",
          headerName: "Defender",
          renderCell: renderTeam,
          minWidth: 200,
          flex: 1,
          sortable: false,
        },
        {
          field: "bot_b",
          headerName: "Challenger",
          renderCell: renderTeam,
          minWidth: 200,
          flex: 1,
          sortable: false,
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
    />
  );
}

function NoTeam() {
  return (
    <Box
      className={secondary_background}
      sx={{
        width: "100%",
        flexGrow: 1,
        padding: "20px",
      }}
    >
      <Container>There is no team at this URL.</Container>
    </Box>
  );
}

export default function ManageTeam({ readonly }: { readonly: boolean }) {
  const user = useUser()[0];

  const team = useTeam()[0];
  if (user === undefined) {
    return <div style={{ flexGrow: 1 }}></div>;
  }
  if (team && user) {
    return (
      <>
        <TeamBar readonly={readonly} />
        <Box
          className={secondary_background}
          sx={{
            width: "100%",
            flexGrow: 1,
            padding: "20px",
          }}
        >
          <Container>
            <h2>Bots</h2>
            {!readonly && <BotUpload />}

            <BotTable readonly={readonly} />
            <h2>Games</h2>
            <GameTable readonly={readonly} />
          </Container>
        </Box>
      </>
    );
  } else if (user) {
    return readonly ? <NoTeam /> : <CreateTeam />;
  } else {
    return <Login />;
  }
}
