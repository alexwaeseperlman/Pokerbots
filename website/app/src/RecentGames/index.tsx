import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Stack, Typography } from "@mui/joy";
import { GameList } from "../components/Tables/GameTable";
import { TeamsTable } from "../components/Tables/TeamsTable";
import BackgroundImage from "../components/BackgroundImage";
import banner from "./banner.jpg";
import HeaderFooter from "../components/HeaderFooter";

export default function RecentGames() {
  return (
    <HeaderFooter>
      <Box
        sx={{
          gridArea: "content",
          display: 'grid'
        }}
      >
        <Typography level="h3" mb={2} color="inherit">
          Recent games
        </Typography>
        <Stack
          sx={{
            width: "100%",
            overflow: "hidden",
            gap: 2,
          }}
        >
          <GameList />
        </Stack>
      </Box>
    </HeaderFooter>
  );
}
