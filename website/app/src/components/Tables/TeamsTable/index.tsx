import React, { useCallback, useEffect } from "react";
import { apiUrl } from "../../../state";
import { Avatar, Box, Chip, ChipProps, Stack, Typography } from "@mui/joy";
import { Link } from "react-router-dom";
import { Team } from "@bindings/Team";
import { TeamsResponse } from "@bindings/TeamsResponse";
import { enqueueSnackbar } from "notistack";
import TeamCard from "./TeamCard";
import { TeamWithMembers } from "@bindings/TeamWithMembers";
import { User } from "@bindings/User";

function getNumberWithOrdinal(n) {
  var s = ["th", "st", "nd", "rd"],
    v = n % 100;
  return n + (s[(v - 20) % 10] || s[v] || s[0]);
}

export function TeamsTable() {
  const [teams, setTeams] = React.useState<TeamWithMembers<User>[]>([]);
  const [loading, setLoading] = React.useState(true);
  const getTeams = useCallback(() => {
    fetch(`${apiUrl}/teams?fill_members=true&sort=Score`)
      .then((res) => res.json())
      .then(async (data: TeamsResponse) => {
        if ("TeamsWithMembers" in data) {
          // swap teama and teamb if teama is not the user's team
          const teams = data;
          setLoading(false);
          setTeams(teams.TeamsWithMembers);
        } else {
          enqueueSnackbar("Error loading teams", { variant: "error" });
        }
      });
  }, []);
  console.log(teams);
  useEffect(() => {
    setLoading(true);
    getTeams();
    const int = setInterval(() => {
      getTeams();
    }, 2000);
    return () => clearInterval(int);
  }, [getTeams]);
  const teamList = teams.map((team, i) => (
    <Box>
      <Typography textColor="#CDC0FF" level="h4" mb={1}>
        {getNumberWithOrdinal(i + 1)} place, rating {team.rating.toFixed(0)}
      </Typography>
      <TeamCard variant={i < 3 ? "large" : "small"} team={team} />
    </Box>
  ));
  return <Box>{teamList}</Box>;
}
