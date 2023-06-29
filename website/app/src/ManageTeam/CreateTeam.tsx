import React, { useState } from "react";
import { useUser } from "../state";
import randomTeamName from "./random-name";
import {
  Box,
  Button,
  Container,
  IconButton,
  Input,
  InputLabel,
  TextField,
  Typography,
} from "@mui/material";
import CasinoIcon from "@mui/icons-material/Casino";
import { Form } from "react-router-dom";

export default function CreateTeam() {
  const user = useUser()[0];
  const [teamName, setTeamName] = useState("");
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        height: "100%",
      }}
    >
      <Container
        sx={{
          display: "flex",
          flexDirection: "column",
          alignItems: "left",
          justifyContent: "center",
          width: "500px",
          background: "white",
          p: 2,
          borderRadius: 2,
        }}
      >
        <form action="/api/create-team">
          <Box sx={{ display: "flex", flexDirection: "column" }}>
            <Typography variant="body1">
              Hi {user?.display_name.split(" ")[0]}. You don't have a team yet.
            </Typography>
            <TextField
              sx={{
                mt: 4,
              }}
              label="Team Name"
              id="team-name"
              variant="standard"
              color="secondary"
              name="team_name"
              value={teamName}
              onChange={(e) => setTeamName(e.target.value)}
              placeholder={randomTeamName()}
              InputProps={{
                endAdornment: (
                  <IconButton onClick={() => setTeamName(randomTeamName())}>
                    <CasinoIcon />
                  </IconButton>
                ),
              }}
            />
            <Button
              color="secondary"
              variant="text"
              sx={{ mt: 4 }}
              type="submit"
            >
              Create
            </Button>
          </Box>
        </form>
      </Container>
    </Box>
  );
}
