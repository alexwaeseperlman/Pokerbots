import { Button, Card, Skeleton, TextField, Typography } from "@mui/material";
import { Box, Container } from "@mui/system";
import React, { useState } from "react";
import { apiUrl, useTeam, useUser } from "../state";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { enqueueSnackbar } from "notistack";
import Login from "../Login";
import { Team } from "@bindings/Team";
import { InviteCodeResponse } from "@bindings/InviteCodeResponse";

export default function JoinTeam() {
  const navigate = useNavigate();
  const code = useSearchParams()[0].get("invite_code");

  const [user, fetchUser] = useUser();
  const [team, setTeam] = useState<Team | null>(null);

  // TODO: Is it actually valid to use an atom family like this?
  const [myTeam, fetchMyTeam] = useTeam(null);

  React.useEffect(() => {
    if (code === undefined) {
      navigate("/");
      enqueueSnackbar("Invalid invite link", { variant: "error" });
    }
    fetch(`${apiUrl}/invite-code?code=${code}`)
      .then((res) => res.json())
      .then((data: InviteCodeResponse | { error: string }) => {
        if ("error" in data) {
          navigate("/");
          enqueueSnackbar(data.error, { variant: "error" });
        }
        setTeam(data.team);
      });
  }, [code]);

  if (!user) {
    return <Login />;
  }

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
          {team?.team_name ? (
            `You are invited to join "${team?.team_name}"`
          ) : (
            <Skeleton
              sx={{
                width: "70vw",
              }}
            />
          )}
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
                if (data) {
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
