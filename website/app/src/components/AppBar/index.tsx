import React from "react";
import {
  menu_bar,
  inner_bar,
  bar_item,
  nav_group,
  bar_item_clickable,
} from "./styles.module.css";
import { useUser } from "../../state";
import Box from "@mui/system/Box";
import Logo from "../Logo";
import IconButton from "@mui/material/IconButton";

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
            transition: "width 0.3s ease-out",
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
  const user = useUser();
  return (
    <Box className={menu_bar}>
      <IconButton>
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
              window.location.href = "/api/signout";
            }}
          />
        )}
      </Box>
    </Box>
  );
}

export function BottomBar() {
  const user = useUser();
  return (
    <Box
      className={menu_bar}
      sx={{
        background:
          "linear-gradient(0deg, rgba(3, 0, 21, 0.58), rgba(3, 0, 21, 0.58)), linear-gradient(89.88deg, #CC385A 0%, #E76FBE 100%)",
      }}
    >
      <Box
        className={nav_group}
        style={{
          flexGrow: 1,
        }}
      >
        <BarItem label="© Poker Zero 2023" />
      </Box>
      <Box className={nav_group}>
        <BarItem label="REPORT AN ISSUE" command={() => {}} />
        <BarItem label="REQUEST A FEATURE" command={() => {}} />
      </Box>
    </Box>
  );
}
