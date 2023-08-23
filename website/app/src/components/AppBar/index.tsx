import React from "react";
import { menu_bar, nav_group, bar_item } from "./styles.module.css";
import { apiUrl, useUser } from "../../state";
import Box from "@mui/joy/Box";
import Logo from "../Logo";
import IconButton from "@mui/joy/IconButton";

import { useNavigate } from "react-router-dom";
import Typography from "@mui/joy/Typography";
import Sheet from "@mui/joy/Sheet";
import { useTheme } from "@mui/joy";

function BarItem(props: {
  label: string;
  selected?: boolean;
  command?: () => void;
}) {
  return (
    <Box onClick={props.command}>
      <Box
        className={bar_item}
        sx={(theme) => ({
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
        })}
      >
        <Typography textColor="inherit" fontWeight={700} level="title-sm">
          {props.label}
        </Typography>
      </Box>
    </Box>
  );
}

export function TopBar() {
  const [user, fetchUser] = useUser();
  const [team, fetchTeam] = useUser();
  const navigate = useNavigate();

  return (
    <Box
      className={`${menu_bar}`}
      sx={(theme) => ({
        // small screen
        [theme.breakpoints.down("sm")]: {
          flexDirection: "column",
          alignItems: "center",
        },
      })}
    >
      <IconButton
        sx={{
          padding: 0,
        }}
        onClick={() => {
          navigate("/");
        }}
      >
        <Logo
          sx={{
            color: "white",
          }}
        />
      </IconButton>
      <BarItem
        label="TEAM"
        selected={window.location.pathname === "/manage-team"}
        command={() => {
          navigate("/manage-team");
        }}
      />
      <BarItem
        label="LEADERBOARD"
        selected={window.location.pathname === "/leaderboard"}
        command={() => {
          navigate("/leaderboard");
        }}
      />

      <BarItem
        label="GAMES"
        selected={window.location.pathname === "/recent_games"}
        command={() => {
          navigate("/recent_games");
        }}
      />

      <Box
        className={nav_group}
        sx={(theme) => ({
          [theme.breakpoints.down("sm")]: {
            display: "none",
          },
          flexGrow: 1,
        })}
      ></Box>
      <BarItem
        label="DOCUMENTATION"
        command={() => {
          window.open("https://docs.upac.dev/");
        }}
      />
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
  );
}

export function BottomBar() {
  return (
    <Box className={menu_bar}>
      <Box
        className={nav_group}
        style={{
          flexGrow: 1,
        }}
      >
        <BarItem label="© UPAC 2023" />
      </Box>
      <BarItem
        label="REPORT AN ISSUE"
        command={() => {
          window.open("https://github.com/alexwaeseperlman/UPAC/issues");
        }}
      />
    </Box>
  );
}
