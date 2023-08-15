import React, { useCallback, useEffect } from "react";
import { apiUrl, useTeam } from "../../state";
import Box from "@mui/system/Box";
import MuiTableCell from "@mui/material/TableCell";
import { styled } from "@mui/material/styles";
import Button, { ButtonProps } from "@mui/material/Button";
import { Avatar, Chip, ChipProps, Typography } from "@mui/material";
import { DataGrid } from "./GameTable";
import { Link } from "react-router-dom";
import { Team } from "@bindings/Team";
import { TeamsResponse } from "@bindings/TeamsResponse";
import { enqueueSnackbar } from "notistack";

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
  const renderTeam = (params: { row: Team }) => {
    console.log(params);
    return (
      <>
        <Avatar
          sx={{
            width: 24,
            height: 24,
            marginRight: 2,
          }}
          src={`${apiUrl}/pfp?id=${params.row?.id}`}
        />
        <Link
          to={`/team/${params.row?.id}`}
          style={{
            color: "inherit",
            textDecoration: "none",
          }}
        >
          <Typography>{params.row?.team_name ?? "Deleted team"}</Typography>
        </Link>
      </>
    );
  };
  return (
    <DataGrid
      columns={[
        {
          field: "score",
          headerName: "Score",
          renderCell: (params) => {
            const score = params.value ?? 0;
            let color: ChipProps["color"] = "success";
            if (score < 0) color = "error";
            else if (score == 0) color = "default";
            return <Chip label={score} color={color} />;
          },
          flex: 1,
          sortable: false,
        },
        {
          field: "team_name",
          headerName: "Team name",
          renderCell: renderTeam,
          flex: 1,
          sortable: false,
        },
      ]}
      loading={loading}
      rows={teams}
      pagination
      pageSizeOptions={[10, 25, 50, 100]}
      paginationMode="server"
      paginationModel={paginationModel}
      rowCount={teamCount ?? 0}
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
