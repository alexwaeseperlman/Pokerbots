import React from "react";
import { Route, Routes, useNavigate } from "react-router-dom";
import HomePage from "./HomePage";
import ManageTeam from "./ManageTeam";
import { useUser } from "./state";

import logoImage from "../static/assets/logo.webp";

function HeaderFooter(props: React.PropsWithChildren<{}>) {
  const user = useUser();
  const navigate = useNavigate();

  return (
    <>
      <body>
        <header>
          <div className="logo" onClick={() => navigate("/")}>
            <img src={logoImage} alt="Logo" />
            <h1>PokerZero</h1>
          </div>

          <nav>
            <a href="/manage-team">Dashboard</a>
            <a href="/leaderboard">Leaderboard</a>
            <a href="https://pokerzero.gitbook.io/pokerzero/" target="_blank">
              Documentation
            </a>
            <a href="mailto:pokerzero3@gmail.com">Contact</a>
          </nav>
          <div className="spacer"></div>
          {user && <a href="/api/signout">Sign out</a>}
        </header>
        {props.children}
        <footer>
          Built with &#9829; by Alex Waese-Perlman, Tommy He, Bonnie Li and
          Santosh Passoubady
        </footer>
      </body>
    </>
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
      </Route>
    </Routes>
  );
}
