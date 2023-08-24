import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";

export default function Leaderboard() {
  return (
    <Box>
      <Card sx={{ p: 2, flexGrow: 1, maxWidth: "100%", mb: 2 }}>
        <Typography level="h3" mb={2}>
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
      </Card>
    </Box>
  );
}