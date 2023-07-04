import React from "react";
import Box from "@mui/system/Box";
import { Grid, Paper, Typography } from "@mui/material";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";

export default function Leaderboard() {
  return (
    <Box
      sx={{
        flexGrow: 1,
        p: 2,
        maxWidth: "100vw",
      }}
    >
      <Paper sx={{ p: 2, flexGrow: 1, maxWidth: "100%", mb: 2 }}>
        <Typography variant="h3" mb={2}>
          Top teams
        </Typography>
        <Box
          sx={{
            width: "100%",
            overflow: "hidden",
          }}
        >
          <TeamsTable />
        </Box>
      </Paper>
      <Paper sx={{ p: 2, flexGrow: 1, maxWidth: "100%", mb: 2 }}>
        <Typography variant="h3" mb={2}>
          Recent games
        </Typography>
        <Box
          sx={{
            width: "100%",
            overflow: "hidden",
          }}
        >
          <GameTable />
        </Box>
      </Paper>
    </Box>
  );
}
