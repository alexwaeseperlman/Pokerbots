import React from "react";
import { useUser, Team, useTeam, User } from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";

import { secondary_background } from "../styles.module.css";
import { TeamBar } from "./TeamBar";
import BotTable from "./BotTable";
import { BotUpload } from "./BotUpload";
import { GameTable } from "./GameTable";

function NoTeam() {
  return (
    <Box
      className={secondary_background}
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
        className={secondary_background}
        sx={{
          width: "100%",
          flexGrow: 1,
          padding: "20px",
        }}
      >
        <Container>
          <h2>Bots</h2>
          {!readonly && <BotUpload />}

          <BotTable readonly={readonly} teamId={teamId} />
          <h2>Games</h2>
          <GameTable teamId={teamId} />
        </Container>
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
  console.log(team, user);
  if (readonly || (team && user)) {
    return <DisplayTeam readonly={readonly} teamId={teamId} />;
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <Login />;
  }
}
