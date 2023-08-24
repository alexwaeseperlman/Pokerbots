import React, { useEffect } from "react";
import { useUser, useTeam } from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";

import { TeamBar } from "./TeamBar";
import BotTable from "./BotTable";
import { BotUpload } from "./BotUpload";
import { GameTable } from "../components/Tables/GameTable";
import Sheet from "@mui/joy/Sheet";
import Stack from "@mui/joy/Stack";
import { Card, CardContent, CardCover, Typography } from "@mui/joy";
import { useNavigate } from "react-router-dom";

function NoTeam() {
  return (
    <Box
      sx={{
        width: "100%",
        flexGrow: 1,
        padding: "20px",
      }}
    >
      <Container>There is no team at this URL.</Container>
    </Box>
  );
}

export function DisplayTeam({
  readonly,
  teamId,
}: {
  readonly: boolean;
  teamId: string | null;
}) {
  const team = useTeam(teamId)[0];
  if (!team) return <NoTeam />;
  return (
    <>
      <TeamBar readonly={readonly} teamId={teamId} />
      <Box
        sx={{
          flexGrow: 1,
        }}
      >
        <Stack gap={2}>
          <Card sx={{ p: 4, pt: 4, mb: 4 }}>
            <CardContent>
              <Typography level="h2">Bots</Typography>
              {!readonly && <BotUpload />}

              <BotTable readonly={readonly} teamId={teamId} />
            </CardContent>
          </Card>
          <Card sx={{ p: 4, pt: 4, mb: 4 }}>
            <Typography level="h2">Games</Typography>
            <GameTable teamId={teamId} />
          </Card>
        </Stack>
      </Box>
    </>
  );
}

export default function ManageTeam({
  teamId,
  readonly,
}: {
  teamId: string | null;
  readonly: boolean;
}) {
  const [team, fetchTeam] = useTeam(teamId);
  const [user, fetchUser] = useUser();
  const navigate = useNavigate();
  useEffect(() => {
    if (!user && !readonly) {
      navigate("/login?redirect=%2Fmanage-team");
    }
  });
  if (readonly || (team && user)) {
    return <DisplayTeam readonly={readonly} teamId={teamId} />;
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <Login />;
  }
}
