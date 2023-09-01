import { Button, Container, Input, Typography } from "@mui/joy";
import { enqueueSnackbar } from "notistack";
import React from "react";
import { authUrl } from "../state";

export default function ForgotPassword() {
  const [email, setEmail] = React.useState("");
  const [loading, setLoading] = React.useState(false);
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
        placeholder="Email"
        type="email"
        value={email}
        onChange={(e) => {
          setEmail(e.target.value);
        }}
      />
      <Button
        variant="solid"
        {...(loading ? { loading: true } : {})}
        onClick={() => {
          setLoading(true);
          fetch(`${authUrl}/email/reset-password`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              email,
              callback: `${window.location.origin}/update-password`,
            }),
          })
            .then(async (res) => {
              if (res.status == 200) {
                enqueueSnackbar("Email sent!", {
                  variant: "success",
                });
              } else {
                enqueueSnackbar(
                  `Failed to send email: ${(await res.json()).error}`,
                  {
                    variant: "error",
                  }
                );
              }
            })
            .finally(() => {
              setLoading(false);
            });
        }}
      >
        Send Email
      </Button>
    </Container>
  );
}
