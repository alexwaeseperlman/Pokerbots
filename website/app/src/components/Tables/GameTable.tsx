import React, { useCallback, useEffect } from "react";
import { apiUrl, usePfpEndpoint, useTeam } from "../../state";
import Box from "@mui/system/Box";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button, { ButtonProps } from "@mui/material/Button";
import {
  Avatar,
  Chip,
  ChipProps,
  IconButton,
  Menu,
  MenuItem,
  Typography,
} from "@mui/material";
import { Link } from "react-router-dom";
import { GridMoreVertIcon } from "@mui/x-data-grid";
import { GameWithBots } from "@bindings/GameWithBots";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { GamesResponse } from "@bindings/GamesResponse";
import { enqueueSnackbar } from "notistack";

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
  type Game = GameWithBots<BotWithTeam<Team>>;
  const [team, fetchTeam] = useTeam(teamId ?? null);
  const [games, setGames] = React.useState<Game[]>([]);
  const [gameCount, setGameCount] = React.useState(0);
  const [pfpEndpoint, fetchPfpEndpoint] = usePfpEndpoint();
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);

  const [menuEl, setMenuEl] = React.useState<null | {
    game: Game;
    el: HTMLElement;
  }>(null);
  const [menuOpen, setMenuOpen] = React.useState(false);

  const getGames = useCallback(() => {
    fetch(
      `${apiUrl}/games?${
        teamId === undefined ? "" : `team=${team?.id}`
      }&count=true`
    )
      .then((res) => res.json())
      .then((data) => setGameCount(data.count));

    fetch(
      `${apiUrl}/games?join_bots=true&page=${paginationModel.page}&page_size=${
        paginationModel.pageSize
      }&${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then(async (data: GamesResponse) => {
        // swap teama and teamb if teama is not the user's team
        setLoading(false);
        console.log(data);
        if ("GamesWithBots" in data) {
          setGames(data.GamesWithBots);
        } else {
          setGames([]);
          enqueueSnackbar("Error loading games", { variant: "error" });
          console.error("Received games as", data);
        }
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
  const renderTeam = (score_mul) => (params) => {
    if (params.value === null) return;
    let color: ChipProps["color"] = "success";
    if (params.row.score_change * score_mul < 0) color = "error";
    else if (params.row.score_change * score_mul == 0) color = "default";
    if (params.row.error_type) {
      color = "warning";
    }
    return (
      <>
        <Avatar
          sx={{
            width: 24,
            height: 24,
            marginRight: 2,
          }}
          src={`${pfpEndpoint}${params.value?.team?.id}`}
        />

        <Chip
          sx={{
            width: "50px !important",
          }}
          label={
            params.row.score_change === null
              ? "Running"
              : params.row.error_type ?? params.row.score_change * score_mul
          }
          color={color}
        />

        <Box ml={2} mr={2} flexDirection={"column"}>
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
  };

  return (
    <>
      <DataGrid
        columns={[
          {
            field: "challenger",
            headerName: "Defender",
            renderCell: renderTeam(-1),
            flex: 1,
            sortable: false,
          },
          {
            field: "defender",
            headerName: "Challenger",
            renderCell: renderTeam(1),
            flex: 1,
            sortable: false,
          },
          {
            field: "options",
            headerName: "",
            width: 40,
            sortable: false,
            renderCell: (params) => {
              let bot = undefined;
              if (params.row.defender?.team?.id === team?.id) {
                bot = params.row.defender?.id;
              } else if (params.row.challenger?.team?.id === team?.id) {
                bot = params.row.challenger?.id;
              }

              const ref = React.createRef<HTMLButtonElement>();

              return (
                <IconButton
                  sx={{
                    color: "black",
                  }}
                  onClick={() => {
                    setMenuEl({
                      game: params.row as Game,
                      el: ref.current!,
                    });
                    setMenuOpen(true);
                  }}
                  ref={ref}
                >
                  <GridMoreVertIcon />
                </IconButton>
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
        rowCount={gameCount ?? 0}
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
      <Menu
        open={menuOpen}
        anchorEl={menuEl?.el}
        onClose={() => setMenuOpen(false)}
        onClick={() => setMenuOpen(false)}
      >
        {team && menuEl?.game?.defender?.team?.id == team.id && (
          <MenuItem
            component="a"
            target="_tab"
            href={`${apiUrl}/game-log?id=${menuEl?.game.id}&bot=${menuEl?.game?.defender.id}`}
          >
            Defender game log
          </MenuItem>
        )}
        {team && menuEl?.game?.challenger?.team?.id == team.id && (
          <MenuItem
            component="a"
            target="_tab"
            href={`${apiUrl}/game-log?id=${menuEl?.game.id}&bot=${menuEl?.game?.challenger.id}`}
          >
            Challenger game log
          </MenuItem>
        )}

        <MenuItem
          component="a"
          target="_tab"
          href={`${apiUrl}/game-log?id=${menuEl?.game.id}`}
        >
          Public game log
        </MenuItem>
      </Menu>
    </>
  );
}
