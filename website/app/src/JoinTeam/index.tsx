import { Button, Card, TextField, Typography } from "@mui/material";
import { Box, Container } from "@mui/system";
import React, { useState } from "react";
import { Team, apiUrl, useTeam } from "../state";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { enqueueSnackbar } from "notistack";

export default function JoinTeam() {
  const navigate = useNavigate();
  const code = useSearchParams()[0].get("invite_code");

  const [team, setTeam] = useState<Team | null>(null);

  // TODO: Is it actually valid to use an atom family like this?
  const [myTeam, fetchMyTeam] = useTeam(null);

  console.log(team, myTeam);
  React.useEffect(() => {
    if (code === undefined) {
      navigate("/");
      enqueueSnackbar("Invalid invite link", { variant: "error" });
    }
    fetch(`${apiUrl}/invite-code?code=${code}`)
      .then((res) => res.json())
      .catch((e) => null)
      .then((data) => setTeam(data[1]));
  }, [code]);

  return (
    <Box
      sx={{
        width: "100%",
        flexGrow: 1,
        padding: "20px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Card
        sx={{
          p: 4,
        }}
      >
        <Typography variant="h3">
          You are invited to join "{team?.team_name}"
        </Typography>
        <Button
          variant="contained"
          color="primary"
          sx={{
            mt: 2,
          }}
          disabled={myTeam !== null}
          onClick={() => {
            fetch(`${apiUrl}/join-team?invite_code=${code}`)
              .then((res) => res.json())
              .then((data) => {
                console.log(data);
                if (data.error) {
                  enqueueSnackbar(data.error, { variant: "error" });
                } else {
                  navigate("/manage-team");
                  enqueueSnackbar("Joined team!", { variant: "success" });
                  fetchMyTeam();
                }
              });
          }}
        >
          {myTeam === null
            ? "Join Team"
            : myTeam.id == team?.id
            ? "Already Joined"
            : "Already on a team. Leave your team to join this one."}
        </Button>
      </Card>
    </Box>
  );
}