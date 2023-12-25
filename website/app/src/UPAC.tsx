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
import VerifyEmail from "./VerifyEmail";
import ForgotPassword from "./ForgotPassword";
import UpdatePassword from "./UpdatePassword";
import OAuth from "./OAuth";
import HeaderFooter from "./components/HeaderFooter";

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

export default function UPAC() {
  return (
    <Routes>
      <Route path="/">
        <Route index element={<Home />} />
        <Route path="manage-team" element={<TeamDashboard />} />
        <Route path="leaderboard" element={<Leaderboard />} />
        <Route path="recent-games" element={<RecentGames />} />
        <Route path="profile" element={<Profile />} />
        <Route path="team">
          <Route path=":teamId" element={<TeamDashboard />} />
        </Route>
        <Route path="join-team" element={<JoinTeam />} />
        <Route path="login" element={<Login />} />
        <Route path="signup" element={<Signup />} />
        <Route path="verify-email/:token" element={<VerifyEmail />} />
        <Route path="forgot-password" element={<ForgotPassword />} />
        <Route path="update-password/:token" element={<UpdatePassword />} />
        <Route path="/login/:provider" element={<OAuth />} />
      </Route>
      <Route path="*" element={<NotFound />} />
    </Routes>
  );
}
