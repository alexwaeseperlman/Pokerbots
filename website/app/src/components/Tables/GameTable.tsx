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
import { enqueueSnackbar } from "notistack";
import { WhichBot } from "@bindings/WhichBot";
import DataTable, { DataTableProps } from "../DataTable";
import { ArrowRight, MoreVert } from "@mui/icons-material";
import { GameWithBotsWithResult } from "@bindings/GameWithBotsWithResult";

export const TableButton = styled((props: ButtonProps) => (
  <Button
    {...props}
    variant="plain"
    sx={{
      color: "#bbb",
      background: "none",
      ":hover": {
        background: "#00000040",
      },
      ":active": {
        background: "#00000080",
      },
    }}
    size="sm"
  />
))(() => ({}));

const RatingChange = ({ before, after }: { before: number; after: number }) => {
  const change = after - before;
  const color = change > 0 ? "success" : change < 0 ? "danger" : "neutral";
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
      }}
    >
      <Chip color={color} size="sm" key="chip">
        {before.toFixed(0)} <ArrowRight /> {after.toFixed(0)}
      </Chip>
    </Box>
  );
};

type Game = GameWithBotsWithResult<BotWithTeam<Team>>;
const renderTeam = ({
  botName,
  whichBot,
  teamId,
  teamName,
}: {
  score: number | null;
  botName: string;
  whichBot: WhichBot;
  teamId: number;
  teamName: string;
}) => {
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: whichBot == "Challenger" ? "row" : "row-reverse",
        alignItems: "center",
        justifyContent: whichBot == "Challenger" ? "right" : "left",
      }}
    >
      <Box
        key="team"
        sx={{
          display: "flex",
          flexDirection: whichBot == "Challenger" ? "row-reverse" : "row",
          alignItems: "center",
        }}
      >
        <Avatar
          key="avatar"
          sx={{
            width: 24,
            height: 24,
          }}
          src={`${apiUrl}/pfp?id=${teamId}`}
        />
        <Box
          key="name"
          ml={2}
          mr={2}
          flexDirection={"column"}
          textAlign={whichBot == "Challenger" ? "right" : "left"}
        >
          <Link
            to={`/team/${teamId}`}
            style={{
              color: "inherit",
              textDecoration: "none",
            }}
          >
            <Typography>{teamName ?? "Deleted team"}</Typography>
          </Link>

          <Typography fontSize="small" textColor="text.secondary">
            {botName ?? "Deleted bot"}
          </Typography>
        </Box>
      </Box>
    </Box>
  );
};

const renderScore = ({
  scoreChange,
  errorType,
  whichBot,
}: {
  scoreChange: number | null;
  errorType: string;
  whichBot: WhichBot;
}) => {
  let color: ChipProps["color"] = "success";
  if (scoreChange == null) color = "warning";
  else if (scoreChange < 0) color = "danger";
  else if (scoreChange == 0) color = "neutral";
  if (errorType) {
    color = "warning";
  }

  return (
    <Box
      sx={{
        flexDirection: "row",
        justifyContent: whichBot == "Challenger" ? "right" : "left",
        display: "flex",
      }}
    >
      <Chip color={color} size="sm" key="chip">
        {scoreChange === null ? "Running" : errorType ?? scoreChange ?? 0}
      </Chip>
    </Box>
  );
};

export function GameTable({ teamId }: { teamId?: string | null }) {
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
      `${apiUrl}/count-games?${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then((data: number | object) => {
        if (typeof data == "number") setGameCount(data);
      });

    fetch(
      `${apiUrl}/games?page=${paginationModel.page}&page_size=${
        paginationModel.pageSize
      }&${teamId === undefined ? "" : `team=${team?.id}`}`
    )
      .then((res) => res.json())
      .then(async (data: GameWithBotsWithResult<BotWithTeam<Team>>[]) => {
        // swap teama and teamb if teama is not the user's team
        setLoading(false);
        if (!("error" in data)) {
          setGames(data);
        } else {
          setGames([]);
          enqueueSnackbar("Error loading games", { variant: "error" });
        }
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

  const columns: DataTableProps<Game>["columns"] = React.useMemo(
    () => [
      {
        name: "",
        key: "challenger rating change",
        width: "75px",
        getProps: (game) => ({
          before: game.challenger_rating,
          after:
            game.challenger_rating +
            (game.result?.challenger_rating_change ?? 0),
          running: !game.result,
        }),
        render: ({ before, after, running }: any) => {
          return running ? (
            <></>
          ) : (
            <RatingChange before={before} after={after} />
          );
        },
      },
      {
        name: "Challenger",
        key: "challenger",
        textAlign: "right",
        getProps: (game) => ({
          botName: game.challenger.name,
          whichBot: "Challenger",
          teamId: game.challenger.team.id,
          teamName: game.challenger.team.name,
        }),
        render: renderTeam,
      },

      {
        name: "",
        key: "challenger score",
        width: "75px",
        getProps: (game) => ({
          whichBot: "Challenger",
          scoreChange: game.result?.challenger_score,
          errorType: game.result?.error_type,
        }),
        render: renderScore,
      },

      {
        name: "",
        key: "defender score",
        width: "75px",
        getProps: (game) => ({
          whichBot: "Defender",
          scoreChange: game.result?.defender_score,
          errorType: game.result?.error_type,
        }),
        render: renderScore,
      },
      {
        name: "Defender",
        key: "defender",
        getProps: (game) => ({
          botName: game.defender.name,
          whichBot: "Defender",
          teamId: game.defender.team.id,
          teamName: game.defender.team.name,
        }),
        render: renderTeam,
      },
      {
        name: "",
        key: "defender rating change",
        width: "75px",
        getProps: (game) => ({
          before: game.defender_rating,
          after:
            game.defender_rating + (game.result?.defender_rating_change ?? 0),
          running: !game.result,
        }),
        render: ({ before, after, running }: any) => {
          return running ? (
            <></>
          ) : (
            <RatingChange before={before} after={after} />
          );
        },
      },
      {
        name: "",
        key: "options",
        width: 40,
        getProps: (game) => ({
          defenderId: game.defender.team.id,
          challengerId: game.challenger.team.id,
          gameId: game.id,
        }),
        render: ({
          defenderId,
          challengerId,
          gameId,
        }: {
          defenderId: number;
          challengerId: number;
          gameId: number;
        }) => {
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
                {defenderId == myTeam?.id && (
                  <MenuItem
                    component="a"
                    target="_tab"
                    href={`${apiUrl}/game-log?id=${gameId}&which_bot=${
                      "Defender" as WhichBot
                    }`}
                  >
                    Defender game log
                  </MenuItem>
                )}
                {challengerId == myTeam?.id && (
                  <MenuItem
                    component="a"
                    target="_tab"
                    href={`${apiUrl}/game-log?id=${gameId}&which_bot=${
                      "Challenger" as WhichBot
                    }`}
                  >
                    Challenger game log
                  </MenuItem>
                )}

                <MenuItem
                  component="a"
                  target="_tab"
                  href={`${apiUrl}/game-log?id=${gameId}`}
                >
                  Public game log
                </MenuItem>
              </Menu>
            </Dropdown>
          );
        },
      },
    ],
    [myTeam?.id]
  );

  return (
    <>
      <DataTable<Game>
        columns={columns}
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
