import React, { Suspense } from "react";
import { Route, Routes, useNavigate, useParams } from "react-router-dom";
import Home from "./Home";
import ManageTeam, { DisplayTeam } from "./ManageTeam";
import { useTeam, useUser } from "./state";

import logoImage from "../static/assets/logo.webp";
import { TopBar, BottomBar } from "./components/AppBar";
import { Box, Container, Sheet } from "@mui/joy";
import Leaderboard from "./Leaderboard";
import { CircularProgress, LinearProgress } from "@mui/joy";
import { useAtom } from "jotai";
import JoinTeam from "./JoinTeam";
import NotFound from "./NotFound";
import RecentGames from "./RecentGames";
import Profile from "./Profile";
import Login from "./Login";
import Signup from "./Signup";
import GameVisualizer from "./GameVisualizer";

function HeaderFooter(props: React.PropsWithChildren<{}>) {
  return (
    <Sheet
      sx={{
        flexDirection: "column",
        minHeight: "100vh",
        position: "relative",
        display: "flex",
        background: "linear-gradient(269.89deg,#392889 0%,#191335 100%)",
        pb: 4,
        boxSizing: "border-box",
      }}
      color="primary"
      variant="solid"
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
        <Container
          sx={{
            margin: "auto",
          }}
        >
          {props.children}
        </Container>
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
    </Sheet>
  );
}

function TeamDashboard() {
  const myTeam = useTeam(null)[0];

  const teamId = useParams().teamId ?? null;

  //TODO: Use suspense here

  return (
    <ManageTeam
      readonly={teamId !== null && myTeam?.id !== parseInt(teamId)}
      teamId={teamId}
    />
  );
}

function GameDashboard(){
  const gameId = useParams().gameId ?? null;
  return (
    <GameVisualizer
      gameId={gameId}
    />
  )
}

export default function UPAC() {
  return (
    <Routes>
      <Route path="/">
        <Route
          index
          element={
            <HeaderFooter>
              <Home />
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
        <Route
          path="recent-games"
          element={
            <HeaderFooter>
              <RecentGames />
            </HeaderFooter>
          }
        />

        <Route
          path="profile"
          element={
            <HeaderFooter>
              <Profile />
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

        <Route
          path="login"
          element={
            <HeaderFooter>
              <Login />
            </HeaderFooter>
          }
        />

        <Route path="view-game">
          <Route
            path=":gameId"
            element={
              <HeaderFooter>
                <GameDashboard />
              </HeaderFooter>
            }
          />
        </Route>

        <Route
          path="signup"
          element={
            <HeaderFooter>
              <Signup />
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
