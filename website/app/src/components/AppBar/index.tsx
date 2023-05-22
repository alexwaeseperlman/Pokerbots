import React from "react";
import {
  menu_bar,
  inner_bar,
  bar_item,
  nav_group,
  bar_item_clickable,
} from "./styles.module.css";
import { apiUrl, useMyTeam, useUser } from "../../state";
import Box from "@mui/system/Box";
import Logo from "../Logo";
import IconButton from "@mui/material/IconButton";

import { primary_background } from "../../styles.module.css";

function BarItem(props: {
  label: string;
  selected?: boolean;
  command?: () => void;
}) {
  return (
    <Box
      className={`${bar_item} ${props.command && bar_item_clickable}`}
      onClick={props.command}
    >
      <Box
        sx={{
          display: "inline-block",
          "::after": {
            content: "''",
            display: "block",
            height: "2px",
            width: props.selected ? "100%" : "0",
            background: "white",
            transition: "width 0.1s ease-out",
          },
          ":hover::after": {
            width: props.command ? "100%" : "0",
          },
        }}
      >
        {props.label}
      </Box>
    </Box>
  );
}

export function TopBar() {
  const [user, fetchUser] = useUser();
  return (
    <Box className={`${menu_bar} ${primary_background}`}>
      <IconButton
        onClick={() => {
          window.location.href = "/";
        }}
      >
        <Logo />
      </IconButton>
      <Box
        className={nav_group}
        style={{
          flexGrow: 1,
        }}
      >
        <BarItem
          label="MANAGE TEAM"
          selected={window.location.pathname === "/manage-team"}
          command={() => {
            window.location.href = "/manage-team";
          }}
        />
        <BarItem
          label="LEADERBOARD"
          selected={window.location.pathname === "/leaderboard"}
          command={() => {
            window.location.href = "/leaderboard";
          }}
        />
      </Box>
      <Box className={nav_group}>
        <BarItem label="DOCUMENTATION" />
        {user && (
          <BarItem
            label="SIGN OUT"
            command={() => {
              fetch(`${apiUrl}/signout`).then(() => {
                fetchUser();
              });
            }}
          />
        )}
      </Box>
    </Box>
  );
}

export function BottomBar() {
  return (
    <Box className={`${menu_bar} ${primary_background}`}>
      <Box
        className={nav_group}
        style={{
          flexGrow: 1,
        }}
      >
        <BarItem label="© Poker Bot League 2023" />
      </Box>
      <Box className={nav_group}>
        <BarItem label="REPORT AN ISSUE" command={() => {}} />
        <BarItem label="REQUEST A FEATURE" command={() => {}} />
      </Box>
    </Box>
  );
}
