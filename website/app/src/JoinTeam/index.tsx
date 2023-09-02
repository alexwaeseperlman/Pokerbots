import { Button, Card, Skeleton, Input, Typography } from "@mui/joy";
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
  const code = useSearchParams()[0].get("code");

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
        if ("error" in data || !("team" in data)) {
          navigate("/");
          enqueueSnackbar(data.error, { variant: "error" });
        } else if ("team" in data) {
          setTeam(data.team);
        }
      });
  }, [code]);

  React.useEffect(() => {
    if (!user) {
      navigate(
        `/login?redirect=${encodeURIComponent("/join-team?code=" + code)}`
      );
    }
  }, [user]);

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
        <Typography level="h3">
          {team?.name ? (
            `You are invited to join "${team?.name}"`
          ) : (
            <Skeleton
              sx={{
                width: "70vw",
              }}
            />
          )}
        </Typography>
        <Button
          variant="plain"
          color="primary"
          sx={{
            mt: 2,
          }}
          disabled={myTeam !== null}
          onClick={() => {
            fetch(`${apiUrl}/join-team?code=${code}`)
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
