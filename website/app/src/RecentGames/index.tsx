import React, { useEffect } from "react";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Stack, Typography } from "@mui/joy";
import { GameList } from "../components/Tables/GameList";
import BackgroundImage from "../components/BackgroundImage";
import banner from "./banner.jpg";
import HeaderFooter from "../components/HeaderFooter";
import { KeyValue } from "../components/KeyValue";

export default function RecentGames() {
  const [gamesCount, setGamesCount] = React.useState(0);
  const [queuedGamesCount, setQueuedGamesCount] = React.useState(0);
  useEffect(() => {
    const getGamesCount = async () => {
      fetch("/api/count-games?running=false")
        .then((res) => res.json())
        .then((res) => {
          setGamesCount(res);
        });
      fetch("/api/count-games?running=true")
        .then((res) => res.json())
        .then((res) => {
          setQueuedGamesCount(res);
        });
    };
    getGamesCount();
    const interval = setInterval(getGamesCount, 1000);
    return () => clearInterval(interval);
  });

  return (
    <HeaderFooter>
      <Box
        sx={{
          gridArea: "head",
          display: "grid",
        }}
      >
        <Typography level="h3" mb={2} color='inherit'>
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
      <Box
        sx={{
          gridArea: "extra",
          display: "grid",
          gridTemplateColumns: "repeat(2, 1fr)",
          height: 'fit-content'
        }}
      >
        <KeyValue keyName="Games played" value={gamesCount.toString()} />
        <KeyValue keyName="Games in queue" value={queuedGamesCount.toString()} />
      </Box>
    </HeaderFooter>
  );
}
