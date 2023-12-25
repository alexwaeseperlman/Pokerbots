import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";
import { GameList } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import banner from "./banner.jpg";
import HeaderFooter from "../components/HeaderFooter";

export default function Leaderboard() {
  return (
    <HeaderFooter>
      <Box sx={{
        gridArea: "content",
        display: 'grid',
        background: 'white'
      }}>
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
      </Box>
    </HeaderFooter>
  );
}
