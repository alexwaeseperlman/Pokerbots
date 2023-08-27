import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import banner from "./banner.jpg";

export default function Leaderboard() {
  return (
    <Box>
      <Card sx={{ p: 2, flexGrow: 1, maxWidth: "100%", mb: 2 }}>
        <Typography level="h3" mb={2}>
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
      </Card>
    </Box>
  );
}
