import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";
import { GameTable } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import BackgroundImage from "../components/BackgroundImage";
import banner from "./banner.jpg";

export default function Leaderboard() {
  return (
    <Box>
      <Card size="lg">
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
