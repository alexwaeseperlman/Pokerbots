import React from "react";
import { apiUrl} from "../state";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Typography } from "@mui/joy";

export default function GameVisualizer({
  gameId,
}: {
  gameId: string | null;
}) {
  const getLogs = useCallback(
    () => {
    fetch(
      `${apiUrl}/game-log?id=${gameId}`
    )
      .then((res) => res.json());
    },
    [gameId],
  )
  return (
    <Box>
      <Card sx={{ p: 2, flexGrow: 1, maxWidth: "100%", mb: 2 }}>
        <Typography level="h3" mb={2}>
          Game {gameId}
        </Typography>
        <Box
          sx={{
            width: "100%",
            overflow: "hidden",
          }}
        >
        </Box>
      </Card>
    </Box>
  );
}
