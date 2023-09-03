import React, { useState } from "react";
import { apiUrl, useTeam, useUser } from "../state";
import randomTeamName from "./random-name";
import {
  Box,
  Button,
  Card,
  Container,
  FormControl,
  FormLabel,
  IconButton,
  Input,
  Sheet,
  Stack,
  Typography,
} from "@mui/joy";
import CasinoIcon from "@mui/icons-material/Casino";
import { Form } from "react-router-dom";
import { enqueueSnackbar } from "notistack";

export default function CreateTeam() {
  const user = useUser()[0];
  const [teamName, setTeamName] = useState("");
  const [team, fetchTeam] = useTeam(null);
  return (
    <Container maxWidth="sm">
      <Card size="lg">
        <Stack gap={2} direction={"column"}>
          <FormControl sx={{ display: "flex", flexDirection: "column" }}>
            <FormLabel>
              Hi {user?.display_name.split(" ")[0]}. You don't have a team yet.
            </FormLabel>
            <Input
              value={teamName}
              onChange={(e) => setTeamName(e.target.value)}
              placeholder={randomTeamName()}
              endDecorator={
                <IconButton onClick={() => setTeamName(randomTeamName())}>
                  <CasinoIcon />
                </IconButton>
              }
            />
          </FormControl>
          <Button
            onClick={() => {
              fetch(
                `${apiUrl}/create-team?name=${encodeURIComponent(teamName)}`
              ).then(async (res) => {
                if (res.status === 200) {
                  fetchTeam();
                } else {
                  const json = await res.json();
                  enqueueSnackbar(json.error, {
                    variant: "error",
                  });
                }
              });
            }}
          >
            Create
          </Button>
        </Stack>
      </Card>
    </Container>
  );
}
