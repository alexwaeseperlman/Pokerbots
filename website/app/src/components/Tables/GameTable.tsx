import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../../state";
import Box from "@mui/system/Box";
import { styled } from "@mui/joy/styles";
import Button, { ButtonProps } from "@mui/joy/Button";
import {
  Avatar,
  Chip,
  ChipProps,
  Dropdown,
  IconButton,
  Menu,
  MenuButton,
  MenuItem,
  Typography,
} from "@mui/joy";
import { Link } from "react-router-dom";
import { GridMoreVertIcon } from "@mui/x-data-grid";
import { GameWithBots } from "@bindings/GameWithBots";
import { BotWithTeam } from "@bindings/BotWithTeam";
import { Team } from "@bindings/Team";
import { GamesResponse } from "@bindings/GamesResponse";
import { enqueueSnackbar } from "notistack";
import { WhichBot } from "@bindings/WhichBot";
import DataTable from "../DataTable";
import { MoreVert } from "@mui/icons-material";

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
  const [myTeam, fetchMyTeam] = useTeam(null);
  const [games, setGames] = React.useState<Game[]>([]);
  const [gameCount, setGameCount] = React.useState(0);
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
      .then((data: GamesResponse) =>
        setGameCount("Count" in data ? Number(data.Count) : 0)
      );

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
  const renderTeam = (whichBot: WhichBot) => (props: { row: Game }) => {
    const scoreMul = whichBot === "Challenger" ? 1 : -1;
    const bot =
      whichBot === "Challenger" ? props.row.challenger : props.row.defender;
    let color: ChipProps["color"] = "success";
    if (props.row.score_change == null) color = "warning";
    else if (props.row.score_change * scoreMul < 0) color = "danger";
    else if (props.row.score_change * scoreMul == 0) color = "neutral";
    if (props.row.error_type) {
      color = "warning";
    }
    return (
      <Box
        sx={{
          display: "flex",
          flexDirection: "row",
        }}
      >
        <Avatar
          sx={{
            width: 24,
            height: 24,
            marginRight: 2,
          }}
          src={`${apiUrl}/pfp?id=${team?.id}`}
        />

        <Box ml={2} mr={2} flexDirection={"column"}>
          <Link
            to={`/team/${bot.team.id}`}
            style={{
              color: "inherit",
              textDecoration: "none",
            }}
          >
            <Typography>{bot.team.team_name ?? "Deleted team"}</Typography>
          </Link>

          <Typography fontSize="small" textColor="text.secondary">
            {bot.name ?? "Deleted bot"}
          </Typography>
        </Box>

        <Chip color={color} size="sm">
          {props.row.score_change === null
            ? "Running"
            : props.row.error_type ?? (props.row.score_change ?? 0) * scoreMul}
        </Chip>
      </Box>
    );
  };

  return (
    <>
      <DataTable<Game>
        columns={[
          {
            name: "Challenger",
            render: renderTeam("Challenger"),
          },
          {
            name: "Defender",
            render: renderTeam("Defender"),
          },
          {
            name: "",
            width: 40,
            render: ({ row: game }) => {
              let bot = undefined;
              const ref = React.createRef<HTMLButtonElement>();
              if (game.defender.team.id === team?.id) {
                bot = game.defender.id;
              } else if (game.challenger.team.id === team?.id) {
                bot = game.challenger.id;
              }

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
                    {team && game.defender.team.id == myTeam?.id && (
                      <MenuItem
                        component="a"
                        target="_tab"
                        href={`${apiUrl}/game-log?id=${game.id}&which_bot=${
                          "Defender" as WhichBot
                        }`}
                      >
                        Defender game log
                      </MenuItem>
                    )}
                    {team && game.challenger.team.id == myTeam?.id && (
                      <MenuItem
                        component="a"
                        target="_tab"
                        href={`${apiUrl}/game-log?id=${game.id}&which_bot=${
                          "Challenger" as WhichBot
                        }`}
                      >
                        Challenger game log
                      </MenuItem>
                    )}

                    <MenuItem
                      component="a"
                      target="_tab"
                      href={`${apiUrl}/game-log?id=${game.id}`}
                    >
                      Public game log
                    </MenuItem>
                  </Menu>
                </Dropdown>
              );
            },
          },
        ]}
        loading={loading}
        data={games}
        total={gameCount ?? 0}
        perPage={paginationModel.pageSize}
        serverPagination={true}
        onPageChange={(page) =>
          setPaginationModel({ ...paginationModel, page })
        }
      />
    </>
  );
}
