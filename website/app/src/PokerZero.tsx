import React, { Suspense } from "react";
import { Route, Routes, useNavigate, useParams } from "react-router-dom";
import HomePage from "./HomePage";
import ManageTeam, { DisplayTeam } from "./ManageTeam";
import { Team, useMyTeam, useTeam, useUser } from "./state";

import logoImage from "../static/assets/logo.webp";
import { TopBar, BottomBar } from "./components/AppBar";
import { Box, Container } from "@mui/system";
import Leaderboard from "./Leaderboard";
import { primary_background } from "./styles.module.css";
import { CircularProgress, LinearProgress } from "@mui/material";
import { useAtom } from "jotai";

function HeaderFooter(props: React.PropsWithChildren<{}>) {
  const user = useUser()[0];
  const navigate = useNavigate();

  return (
    <Box
      display={"flex"}
      flexDirection={"column"}
      height="100%"
      className={primary_background}
    >
      <TopBar />
      <Suspense
        fallback={
          <>
            <Box
              sx={{
                flexGrow: 1,
                display: "flex",
                justifyContent: "center",
                alignItems: "center",
              }}
            >
              <CircularProgress />
            </Box>
          </>
        }
      >
        {props.children}
      </Suspense>

      <BottomBar />
    </Box>
  );
}

function TeamDashboard() {
  const myTeam = useTeam(null)[0];

  const teamId = useParams().teamId ?? null;

  const user = useUser()[0];

  //TODO: Use suspense here
  console.log(teamId);

  return (
    <ManageTeam
      readonly={teamId !== null && myTeam?.id !== parseInt(teamId)}
      teamId={teamId}
    />
  );
}

export default function PokerZero() {
  return (
    <Routes>
      <Route path="/">
        <Route
          index
          element={
            <HeaderFooter>
              <HomePage />
            </HeaderFooter>
          }
        />
        <Route
          path="manage-team"
          element={
            <HeaderFooter>
              <TeamDashboard />
            </HeaderFooter>
          }
        />
        <Route
          path="leaderboard"
          element={
            <HeaderFooter>
              <Leaderboard />
            </HeaderFooter>
          }
        />
        <Route path="team">
          <Route
            path=":teamId"
            element={
              <HeaderFooter>
                <TeamDashboard />
              </HeaderFooter>
            }
          />
        </Route>
      </Route>
    </Routes>
  );
}
