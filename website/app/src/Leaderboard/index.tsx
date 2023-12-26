import React from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";
import banner from "./banner.jpg";
import HeaderFooter from "../components/HeaderFooter";
import { TeamsTable } from "../components/Tables/TeamsTable";

export default function Leaderboard() {
  return (
    <HeaderFooter>
      <Box sx={{
        gridArea: "content",
        display: 'grid',
      }}>
        <TeamsTable/>
      </Box>
    </HeaderFooter>
  );
}
