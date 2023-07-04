import React from "react";
import Box from "@mui/system/Box";
import { Grid, Paper, Typography } from "@mui/material";
import { GameTable } from "../ManageTeam/GameTable";

export default function Leaderboard() {
  return (
    <Box
      sx={{
        flexGrow: 1,
        display: "flex",
        flexDirection: "row",
        flexWrap: "wrap",
        gap: 2,
        p: 2,
      }}
    >
      <Paper sx={{ p: 2, flexGrow: 1 }}>
        <Typography variant="h3" mb={2}>
          Top teams
        </Typography>
        <Box>
          <GameTable teamId={null} />
        </Box>
      </Paper>
      <Paper sx={{ p: 2, flexGrow: 1 }}>
        <Typography variant="h3" mb={2}>
          Recent games
        </Typography>
        <Box>
          <GameTable />
        </Box>
      </Paper>
    </Box>
  );
}
