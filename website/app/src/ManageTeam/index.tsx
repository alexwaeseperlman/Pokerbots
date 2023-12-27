import React, { useEffect } from "react";
import { useUser, useTeam, apiUrl, useProfile } from "../state";
import CreateTeam from "./CreateTeam";
import Login from "../Login";
import Box from "@mui/system/Box";
import { Container } from "@mui/system";
import { team_member_table_row } from "./styles.module.css";

import { TeamBar } from "./TeamBar";
import BotTable from "./BotTable";
import FileUpload from "../components/BotUpload";
import { GameList } from "../components/Tables/GameTable";
import Sheet from "@mui/joy/Sheet";
import Stack from "@mui/joy/Stack";
import { Card, CardContent, CardCover, Typography } from "@mui/joy";
import { useNavigate } from "react-router-dom";
import { enqueueSnackbar } from "notistack";
import NoProfile from "./NoProfile";
import HeaderFooter from "../components/HeaderFooter";
import BotList from "./BotList";
import { ReadOnlyProvider, useReadOnly } from "./state";
import { KeyValue } from "../components/KeyValue";
import { TeamWithMembers } from "@bindings/TeamWithMembers";
import { User } from "@bindings/User";

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

// TODO: polling for team rank is incredibly inefficient
function TeamStats({ team }: { team: TeamWithMembers<User> }) {
  const [gamesPlayed, setGamesPlayed] = React.useState<number | null>(null);
  const [teamRank, setTeamRank] = React.useState<number | null>(null);

  useEffect(() => {
    const getGames = () => {
      fetch(`${apiUrl}/count-games?team_id=${team.id}`).then(async (res) => {
        const json = await res.json();
        if (res.status === 200) {
          setGamesPlayed(json);
        } else {
          console.log(json);
        }
      });
    };
    getGames();
    const interval = setInterval(getGames, 1000);
    return () => clearInterval(interval);
  }, [team.id]);

  useEffect(() => {
    const getRank = () => {
      fetch(`${apiUrl}/teams?sort=Score`).then(async (res) => {
        const json = await res.json();
        if (res.status === 200) {
          const teams = json.Teams;
          const teamRank = teams.findIndex((t) => t.id === team.id) + 1;
          setTeamRank(teamRank);
        } else {
          console.log(json);
        }
      });
    }
    getRank();
    const interval = setInterval(getRank, 1000);
    return () => clearInterval(interval);
  }, [team.id]);

  return (
    <Box
      sx={{
        display: "grid",
        height: "fit-content",
        gridTemplateColumns: "repeat(2, 1fr)",
      }}
    >
      <KeyValue keyName="Rank" value={teamRank} />
      <KeyValue keyName="Rating" value={team.rating.toFixed(0)} />
      <KeyValue keyName="Games played" value={gamesPlayed} />
      <KeyValue keyName="Team ID" value={team.id.toString()} />
    </Box>
  );
}

export function DisplayTeam({ teamId }: { teamId: string | null }) {
  const team = useTeam(teamId)[0];
  const isReadOnly = useReadOnly();
  if (!team) return <NoTeam />;
  return (
    <>
      <Box
        sx={{
          display: "grid",
          gridArea: "extra",
        }}
      >
        <TeamStats team={team} />
      </Box>
      <Box
        sx={{
          display: "grid",
          gridArea: "head",
        }}
      >
        <TeamBar teamId={teamId} />
      </Box>
      <Box
        sx={{
          display: "grid",
          gridArea: "content",
        }}
      >
        <Stack gap={2}>
          <Typography level="h2" color="inherit">
            Bots
          </Typography>
          {team.active_bot == null && (
            <Typography color="danger">
              This team doesn't have an active bot.{" "}
              {!isReadOnly && "Upload a bot to start playing."}
            </Typography>
          )}
          {!isReadOnly && (
            <FileUpload onUpload={handleUpload}>
              Drag a zipped bot here
            </FileUpload>
          )}

          <BotList teamId={teamId} />
          <Typography level="h2" color="inherit">
            Games
          </Typography>
          <GameList teamId={teamId} />
        </Stack>
      </Box>
    </>
  );
  function handleUpload(file: File) {
    return fetch(`${apiUrl}/upload-bot`, {
      method: "POST",
      body: file,
    }).then(async (res) => {
      const json = await res.json();
      if (res.status !== 200) {
        enqueueSnackbar({
          message: json.error,
          variant: "error",
        });
      }
    });
  }
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
  const [profile, fetchProfile] = useProfile();
  const navigate = useNavigate();
  useEffect(() => {
    if (!user && !readonly) {
      navigate("/login?redirect=%2Fmanage-team");
    }
  });
  const render = () => {
    if (readonly || (team && user)) {
      return (
        <ReadOnlyProvider isReadOnly={readonly}>
          <DisplayTeam teamId={teamId} />
        </ReadOnlyProvider>
      );
    } else if (user && !profile) {
      return (
        <Box
          sx={{
            gridArea: "content",
            display: "flex",
            alignItems: "center",
            justifyContent: "stretch",
          }}
        >
          <NoProfile />
        </Box>
      );
    } else if (user) {
      return (
        <Box
          sx={{
            gridArea: "content",
            display: "flex",
            alignItems: "center",
            justifyContent: "stretch",
          }}
        >
          <CreateTeam />
        </Box>
      );
    } else {
      return (
        <Box
          sx={{
            gridArea: "content",
            display: "flex",
            alignItems: "center",
            justifyContent: "stretch",
          }}
        >
          <Login />
        </Box>
      );
    }
  };
  return <HeaderFooter>{render()}</HeaderFooter>;
}
