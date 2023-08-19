import React, { Suspense } from "react";
import { Route, Routes, useNavigate, useParams } from "react-router-dom";
import HomePage from "./HomePage";
import ManageTeam, { DisplayTeam } from "./ManageTeam";
import { useTeam, useUser } from "./state";

import logoImage from "../static/assets/logo.webp";
import { TopBar, BottomBar } from "./components/AppBar";
import { Box, Container } from "@mui/system";
import Leaderboard from "./Leaderboard";
import { primary_background } from "./styles.module.css";
import { CircularProgress, LinearProgress } from "@mui/material";
import { useAtom } from "jotai";
import JoinTeam from "./JoinTeam";
import NotFound from "./NotFound";

function HeaderFooter(props: React.PropsWithChildren<{}>) {
  const user = useUser()[0];
  const navigate = useNavigate();

  return (
    <Box
      flexDirection={"column"}
      minHeight="100vh"
      position="relative"
      display={"flex"}
      className={primary_background}
      pb={4}
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

      <Box
        sx={{
          position: "absolute",
          bottom: 0,
          width: "100%",
        }}
      >
        <BottomBar />
      </Box>
    </Box>
  );
}

function TeamDashboard() {
  const myTeam = useTeam(null)[0];

  const teamId = useParams().teamId ?? null;

  const user = useUser()[0];

  //TODO: Use suspense here

  return (
    <ManageTeam
      readonly={teamId !== null && myTeam?.id !== parseInt(teamId)}
      teamId={teamId}
    />
  );
}

export default function UPAC() {
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

        <Route
          path="join-team"
          element={
            <HeaderFooter>
              <JoinTeam />
            </HeaderFooter>
          }
        />
      </Route>
      <Route
        path="*"
        element={
          <HeaderFooter>
            <NotFound />
          </HeaderFooter>
        }
      />
    </Routes>
  );
}
