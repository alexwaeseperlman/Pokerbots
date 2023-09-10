import React, { useCallback, useEffect, useState } from "react";
import { apiUrl } from "../state";
import Box from "@mui/joy/Box";
import { Card, Grid, Sheet, Slider, Typography, Input } from "@mui/joy";

interface GameState {
  game: string,
  step: number,
  challenger_stack: number,
  defender_stack: number,
  challenger_pushed: number,
  defender_pushed: number,
  pot: number,
  challenger_hand: string,
  defender_hand: string,
  flop: string,
  turn: string,
  river: string,
  button: string,
  round: string,
  last_action: string,
}

function GetGameState({ gameId, step }: { gameId: string, step: number; }) {

  const [logs, setLogs] = useState<GameState>()

  const fetchData = () => {

    fetch(
      `${apiUrl}/game-state?id=${gameId}&state=${step}`)
      .then(res => { return res.json() })
      .then(data => { setLogs(data) }
      )
  }

  useEffect(() => fetchData(), [step])
  return (
    <Grid
      container
      sx={{ flexGrow: 1 }}
      alignItems="center"
    >
      <Grid xs={12} style={{ fontSize: 23, textAlign: "center" }}>
        {logs ? logs["challenger_stack"] : ""}
      </Grid>
      <Grid xs={12} >
        <Typography style={{ fontSize: 100, textAlign: "center" }}>
          {logs ? logs["challenger_hand"] : ""}
        </Typography>
      </Grid>
      <Grid xs={12} style={{ fontSize: 25, textAlign: "center" }} >
        {logs ? logs["challenger_pushed"] : ""}
      </Grid>
      <Grid xs={12} style={{ fontSize: 100, textAlign: "center" }}  >
        {logs ? logs["flop"] : " "}
        {" "}
        {logs ? logs["turn"] : " "}
        {" "}
        {logs ? logs["river"] : " "}
      </Grid>
      <Grid xs={12} style={{ fontSize: 25, textAlign: "center" }} >
        {logs ? logs["defender_pushed"] : ""}
      </Grid>
      <Grid xs={12} style={{ fontSize: 100, textAlign: "center" }}>
        {logs ? logs["defender_hand"] : " "}
      </Grid>
      <Grid xs={12} style={{ fontSize: 23, textAlign: "center" }}>
        {logs ? logs["defender_stack"] : ""}
      </Grid>
      {logs?.round}
      {/* <Card */}
      {/*   sx={{ */}
      {/*     textAlign: 'center', */}
      {/*     alignItems: 'center', */}
      {/*     width: 20, */}
      {/*     height: 35, */}
      {/*     fontSize: 23, */}
      {/*   }} */}
      {/* > */}
      {/* {[String.fromCharCode(0x2665), 10].join("")} */}
      {/* </Card>  */}
      {/* <Card */}
      {/*   sx={{ */}
      {/*     textAlign: 'center', */}
      {/*     alignItems: 'center', */}
      {/*     width: 20, */}
      {/*     height: 35, */}
      {/*     fontSize: 23, */}
      {/*   }} */}
      {/* > */}
      {/* {[String.fromCharCode(0x2665), 5].join("")} */}
      {/* </Card> */}
    </Grid >

  )
}

export default function GameVisualizer({
  gameId,
}: {
  gameId: string;
}) {
  const [max, setMax] = useState(0)
  const [inputValue, setInputValue] = useState(0);
  const handleInputChange = (event: any) => {
    setInputValue(parseInt(event.target.value) || 0);
  }
  useEffect(
    () => {
      if (max == 0)
        fetch(`${apiUrl}/game-length?game=${gameId}`).then(data => data.text()).then(data => setMax(parseInt(data)));
    },
    [max],
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
        <GetGameState gameId={gameId} step={inputValue} />
      </Card>
      <Input type="number" value={inputValue} onChange={handleInputChange} sx={{ width: "10%" }} slotProps={{
        input: {
          min: 0,
          max: max,
        },
      }} />
      <Slider
        value={inputValue}
        onChange={handleInputChange}
        step={1}
        marks
        min={0}
        max={max} />
    </Box>
  );
}
