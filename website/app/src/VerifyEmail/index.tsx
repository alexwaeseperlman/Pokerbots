import { Button, CircularProgress, Container, Typography } from "@mui/joy";
import { enqueueSnackbar } from "notistack";
import React from "react";
import { useNavigate, useParams } from "react-router-dom";
import { authUrl } from "../state";

export default function VerifyEmail() {
  const params = useParams();
  const navigate = useNavigate();
  React.useEffect(() => {
    fetch(`${authUrl}/email/verify/${params.token}`, {
      method: "POST",
    }).then(async (res) => {
      if (res.status == 200) {
        enqueueSnackbar("Email verified!", {
          variant: "success",
        });
        navigate("/login");
      } else {
        enqueueSnackbar(`Failed to verify email: ${(await res.json()).error}`, {
          variant: "error",
        });
        navigate("/");
      }
    });
  }, [params.token]);
  return (
    <Container
      maxWidth="sm"
      sx={{
        display: "flex",
        flexDirection: "column",
        alignItems: "stretch",
        justifyContent: "center",
        flexGrow: 1,
        gap: 2,
      }}
    >
      <CircularProgress />
    </Container>
  );
}
