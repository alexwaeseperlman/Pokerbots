import { Button, Container, Input, Typography } from "@mui/joy";
import { enqueueSnackbar } from "notistack";
import React from "react";
import { authUrl } from "../state";
import { useNavigate, useParams } from "react-router-dom";

export default function UpdatePassword() {
  const params = useParams();
  const [password, setPassword] = React.useState("");
  const [confirmPassword, setConfirmPassword] = React.useState("");
  const [loading, setLoading] = React.useState(false);
  const navigate = useNavigate();
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
      <Typography textColor="inherit" level="h1">
        Forgot Password
      </Typography>
      <Input
        placeholder="Password"
        type="password"
        value={password}
        onChange={(e) => {
          setPassword(e.target.value);
        }}
      />
      <Input
        placeholder="Confirm your password"
        type="password"
        value={confirmPassword}
        onChange={(e) => {
          setConfirmPassword(e.target.value);
        }}
      />

      <Button
        variant="solid"
        {...(loading ? { loading: true } : {})}
        onClick={() => {
          setLoading(true);
          fetch(`${authUrl}/email/reset-password/${params.token}`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              password,
            }),
          })
            .then(async (res) => {
              if (res.status == 200) {
                enqueueSnackbar("Reset password", {
                  variant: "success",
                });
                navigate("/login");
              } else {
                enqueueSnackbar(
                  `Failed to update password: ${(await res.json()).error}`,
                  {
                    variant: "error",
                  }
                );
                navigate("/");
              }
            })
            .finally(() => {
              setLoading(false);
            });
        }}
      >
        Update
      </Button>
    </Container>
  );
}
