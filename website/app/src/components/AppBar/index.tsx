import React, { PropsWithChildren } from "react";
import { menu_bar, nav_group, bar_item } from "./styles.module.css";
import { apiUrl, useUser } from "../../state";
import Box from "@mui/joy/Box";
import Logo from "../Logo";
import IconButton from "@mui/joy/IconButton";

import { useNavigate, useSearchParams } from "react-router-dom";
import Typography from "@mui/joy/Typography";
import Sheet from "@mui/joy/Sheet";
import { useTheme } from "@mui/joy";
import { Person } from "@mui/icons-material";

function RawBarItem(
  props: PropsWithChildren<{ selected?: boolean; command?: () => void }>
) {
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
        {props.children}
      </Box>
    </Box>
  );
}

function BarItem(props: {
  label?: string;
  selected?: boolean;
  command?: () => void;
}) {
  return (
    <RawBarItem selected={props.selected} command={props.command}>
      <Typography textColor="inherit" fontWeight={700} level="title-sm">
        {props.label}
      </Typography>
    </RawBarItem>
  );
}

export function TopBar() {
  const [user, fetchUser] = useUser();
  const [team, fetchTeam] = useUser();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

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
        color="neutral"
        variant="solid"
        sx={{
          padding: 0,
          background: "none",
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
        selected={window.location.pathname === "/recent-games"}
        command={() => {
          navigate("/recent-games");
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
      {user ? (
        <>
          <BarItem
            label="SIGN OUT"
            command={() => {
              fetch(`${apiUrl}/signout`).then(() => {
                fetchUser();
              });
            }}
          />
          <RawBarItem
            selected={window.location.pathname === "/profile"}
            command={() => {
              navigate("/profile");
            }}
          >
            <Person />
          </RawBarItem>
        </>
      ) : (
        <>
          <BarItem
            label="JOIN!"
            selected={window.location.pathname === "/signup"}
            command={() => {
              navigate("/signup");
            }}
          />
        </>
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
