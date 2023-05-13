import React from "react";
import { Route, Routes, useNavigate } from "react-router-dom";
import HomePage from "./HomePage";
import ManageTeam from "./ManageTeam";
import { useUser } from "./state";

import logoImage from "../static/assets/logo.webp";
import { TopBar, BottomBar } from "./components/AppBar";
import { Box, Container } from "@mui/system";
import Leaderboard from "./Leaderboard";
import { primary_background } from "./styles.module.css";

function HeaderFooter(props: React.PropsWithChildren<{}>) {
  const user = useUser();
  const navigate = useNavigate();

  return (
    <Box
      display={"flex"}
      flexDirection={"column"}
      height="100%"
      className={primary_background}
    >
      <TopBar />
      {props.children}

      <BottomBar />
    </Box>
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
              <ManageTeam />
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
      </Route>
    </Routes>
  );
}
