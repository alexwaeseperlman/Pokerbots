import React, { useCallback, useEffect } from "react";
import {
  Game,
  apiUrl,
  useTeam,
  useUser,
  Team,
  pfpEndpoint,
  fillInGames,
} from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button from "@mui/material/Button";

import { secondary_background } from "../styles.module.css";
import { TeamBar } from "./TeamBar";
import { Avatar, Chip, Typography } from "@mui/material";
import BotTable from "./BotTable";

const DataGrid = React.lazy(() =>
  import("@mui/x-data-grid").then((mod) => ({ default: mod.DataGrid }))
);

export const TableCell = styled(MuiTableCell)({
  borderBottom: "none",
});
export const TableButton = styled((props) => (
  <Button {...props} disableRipple />
))({
  fontSize: "12px",
  fontWeight: 300,
  textAlign: "left",
  justifyContent: "left",
  textTransform: "none",
  padding: 0,
  cursor: "pointer",
});

function GameTable() {
  const team = useTeam()[0];
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
        const games = data.map((game) =>
          game.teama != team?.id
            ? game
            : {
                ...game,
                bot_a: game.bot_a,
                bot_b: game.bot_b,
                score_change:
                  game.score_change === null ? null : -game.score_change,
              }
        );

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
    }, 1000);
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

        <Typography color={"text.secondary"}>{params.value?.name}</Typography>
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
            let color = "success";
            if (params.value < 0) color = "error";
            else if (params.value == 0) color = "default";
            return <Chip label={params.value} color={color} />;
          },
          minWidth: 100,
          flex: 1,
        },
        {
          field: "bot_a",
          headerName: "You",
          renderCell: renderTeam,
          minWidth: 200,
          flex: 1,
        },
        {
          field: "bot_b",
          headerName: "Opponent",
          renderCell: renderTeam,
          minWidth: 200,
          flex: 1,
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
    />
  );
}

export default function ManageTeam() {
  const user = useUser()[0];
  const team = useTeam()[0];

  if (user === undefined) {
    return <div style={{ flexGrow: 1 }}></div>;
  }
  if (team && user) {
    return (
      <>
        <TeamBar />
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

            <BotTable />
            <h2>Games</h2>
            <GameTable />
          </Container>
        </Box>
      </>
    );
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <Login />;
  }
}
