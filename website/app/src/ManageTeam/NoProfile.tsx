import React, { useState } from "react";
import {
  Button,
  Card,
  Container,
  FormControl,
  FormLabel,
  Stack,
} from "@mui/joy";
import { useNavigate } from "react-router-dom";

export default function NoProfile() {
  const navigate = useNavigate();
  return (
    <Container maxWidth="sm">
      <Card size="lg">
        <Stack gap={2} direction={"column"}>
          <FormControl sx={{ display: "flex", flexDirection: "column" }}>
            <FormLabel>
              You need to complete your profile before you can create a team.
            </FormLabel>
          </FormControl>
          <Button
            onClick={() => {
              navigate("/profile");
            }}
          >
            Complete your profile
          </Button>
        </Stack>
      </Card>
    </Container>
  );
}
