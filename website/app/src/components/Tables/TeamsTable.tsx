import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../../state";
import Box from "@mui/system/Box";
import { styled } from "@mui/joy/styles";
import Button, { ButtonProps } from "@mui/joy/Button";
import { Avatar, Chip, ChipProps, Typography } from "@mui/joy";
import { Link } from "react-router-dom";
import { Team } from "@bindings/Team";
import { TeamsResponse } from "@bindings/TeamsResponse";
import { enqueueSnackbar } from "notistack";
import DataTable from "../DataTable";

export function TeamsTable() {
  const [teams, setTeams] = React.useState<Team[]>([]);
  const [teamCount, setTeamCount] = React.useState(0);
  const [paginationModel, setPaginationModel] = React.useState({
    page: 0,
    pageSize: 10,
  });
  const [loading, setLoading] = React.useState(true);
  const getTeams = useCallback(() => {
    fetch(`${apiUrl}/teams?count=true`)
      .then((res) => res.json())
      .then((data: TeamsResponse) =>
        setTeamCount("Count" in data ? Number(data.Count) : 0)
      );

    fetch(
      `${apiUrl}/teams?page=${paginationModel.page}&page_size=${paginationModel.pageSize}`
    )
      .then((res) => res.json())
      .then(async (data: TeamsResponse) => {
        if ("Teams" in data) {
          // swap teama and teamb if teama is not the user's team
          const teams = data;
          setLoading(false);
          setTeams(teams.Teams);
        } else {
          enqueueSnackbar("Error loading teams", { variant: "error" });
          console.error("Received teams as", data);
        }
      });
  }, [paginationModel.page, paginationModel.pageSize]);
  useEffect(() => {
    setLoading(true);
    getTeams();
  }, [getTeams, paginationModel]);
  const renderTeam = ({
    teamId,
    teamName,
  }: {
    teamId: string;
    teamName: string;
  }) => {
    return (
      <>
        <Avatar
          sx={{
            width: 24,
            height: 24,
            marginRight: 2,
          }}
          src={`${apiUrl}/pfp?id=${teamId}`}
        />
        <Link
          to={`/team/${teamId}`}
          style={{
            color: "inherit",
            textDecoration: "none",
          }}
        >
          <Typography>{teamName ?? "Deleted team"}</Typography>
        </Link>
      </>
    );
  };
  return (
    <DataTable<Team>
      columns={[
        {
          name: "Score",
          width: "100px",
          getProps: (team) => ({ score: team.score }),
          render: ({ score }: { score: number | null }) => {
            let color: ChipProps["color"] = "success";
            if ((score ?? 0) == 0) color = "neutral";
            else if ((score ?? 0) < 0) color = "danger";
            return <Chip color={color}>{score ?? 0}</Chip>;
          },
        },
        {
          name: "Team name",
          getProps: (team) => ({ teamId: team.id, teamName: team.team_name }),
          render: renderTeam,
        },
      ]}
      loading={loading}
      data={teams}
      perPage={paginationModel.pageSize}
      serverPagination
      total={teamCount ?? 0}
      onPageChange={(page) => setPaginationModel({ ...paginationModel, page })}
    />
  );
}
