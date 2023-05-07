import React from "react";
import { Route, Routes } from "react-router-dom";
import HomePage from "./HomePage";
import ManageTeam from "./ManageTeam";

export default function PokerZero() {
  return (
    <Routes>
      <Route path="/">
        <Route index element={<HomePage />} />
        <Route path="manage-team" element={<ManageTeam />} />
      </Route>
    </Routes>
  );
}
