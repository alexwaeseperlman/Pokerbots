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
import { BoxProps } from "@mui/joy/Box";

function RawBarItem({
  selected,
  command,
  children,
  ...props
}: PropsWithChildren<{ selected?: boolean; command?: () => void } & BoxProps>) {
  return (
    <Box
      onClick={command}
      {...props}
      onKeyDown={(e) => {
        if (e.key == " " || e.key == "Enter") {
          e.preventDefault();
          return (e.key == " " || e.key == "Enter") && command?.();
        }
      }}
    >
      <Box
        className={bar_item}
        sx={(theme) => ({
          "::after": {
            content: "''",
            display: "block",
            height: "2px",
            width: selected ? "100%" : "0",
            background: "white",
            transition: "width 0.1s ease-out",
          },
          ":hover::after": {
            width: command ? "100%" : "0",
          },
        })}
      >
        {children}
      </Box>
    </Box>
  );
}

function BarItem({
  label,
  selected,
  command,
  ...props
}: {
  label?: string;
  selected?: boolean;
  command?: () => void;
} & BoxProps) {
  return (
    <RawBarItem {...props} selected={selected} command={command}>
      <Typography textColor="inherit" fontWeight={700} level="title-sm">
        {label}
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
          alignItems: "baseline",
        },
      })}
    >
      <RawBarItem
        tabIndex={1}
        command={() => {
          navigate("/");
        }}
        sx={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          pt: 1,
        }}
      >
        <Logo
          sx={{
            color: "inherit",
          }}
        />
      </RawBarItem>
      {user && (
        <BarItem
          tabIndex={2}
          label="YOUR TEAM"
          selected={window.location.pathname === "/manage-team"}
          command={() => {
            navigate("/manage-team");
          }}
        />
      )}
      <BarItem
        tabIndex={3}
        label="LEADERBOARD"
        selected={window.location.pathname === "/leaderboard"}
        command={() => {
          navigate("/leaderboard");
        }}
      />

      <BarItem
        tabIndex={4}
        label="GAMES"
        selected={window.location.pathname === "/recent-games"}
        command={() => {
          navigate("/recent-games");
        }}
      />

      <Box
        tabIndex={5}
        className={nav_group}
        sx={(theme) => ({
          [theme.breakpoints.down("sm")]: {
            display: "none",
          },
          flexGrow: 1,
        })}
      ></Box>
      <BarItem
        tabIndex={6}
        label="DOCUMENTATION"
        command={() => {
          window.open("https://docs.upac.dev/");
        }}
      />
      {user ? (
        <>
          <BarItem
            tabIndex={7}
            label="SIGN OUT"
            command={() => {
              fetch(`${apiUrl}/signout`).then(() => {
                fetchUser();
              });
            }}
          />
          <RawBarItem
            tabIndex={8}
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
            tabIndex={7}
            label="LOG IN"
            selected={window.location.pathname === "/login"}
            command={() => {
              navigate("/login");
            }}
          />
          <BarItem
            tabIndex={8}
            label="SIGN UP"
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
