import React, { PropsWithChildren } from "react";
import { menu_bar, nav_group, bar_item } from "./styles.module.css";
import { apiUrl, authUrl, useAuth, useUser } from "../../state";
import Box from "@mui/joy/Box";
import Logo from "../Logo";
import IconButton from "@mui/joy/IconButton";
import { useNavigate, useSearchParams } from "react-router-dom";
import Typography from "@mui/joy/Typography";
import Sheet from "@mui/joy/Sheet";
import {
  Accordion,
  AccordionDetails,
  AccordionGroup,
  AccordionSummary,
  Badge,
  Dropdown,
  Menu,
  MenuButton,
  useTheme,
} from "@mui/joy";
import { MenuOpen, Person, Menu as MenuIcon } from "@mui/icons-material";
import { BoxProps } from "@mui/joy/Box";

function RawBarItem({
  selected,
  command,
  underlineColor = "white",
  children,
  ...props
}: PropsWithChildren<
  {
    selected?: boolean;
    underlineColor?: "white" | "black";
    command?: () => void;
  } & BoxProps
>) {
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
            background: underlineColor,
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
  underlineColor = "white",
  ...props
}: {
  label?: string;
  selected?: boolean;
  underlineColor?: "white" | "black";
  command?: () => void;
} & BoxProps) {
  return (
    <RawBarItem
      {...props}
      underlineColor={underlineColor}
      selected={selected}
      command={command}
    >
      <Typography textColor="inherit" fontWeight={700} level="title-sm">
        {label}
      </Typography>
    </RawBarItem>
  );
}

export function TopBar() {
  return (
    <>
      <Box>
        <Dropdown
          sx={(theme) => ({
            [theme.breakpoints.up("sm")]: {
              display: "none",
            },
          })}
        >
          <MenuButton
            sx={(theme) => ({
              [theme.breakpoints.up("sm")]: {
                display: "none",
              },
              background: "none",
              border: "none",
              ":hover": {
                background: "#00000011",
              },
              justifyContent: "flex-start",
              alignItems: "center",
            })}
            color='primary'
            variant="solid"
          >
            <MenuIcon sx={{ color: "inherit" }} />
          </MenuButton>
          <Menu
            sx={(theme) => ({
              [theme.breakpoints.up("sm")]: {
                display: "none",
              },
            })}
          >
            <TopBarContent vertical black={true} />
          </Menu>
        </Dropdown>
      </Box>
      <Box
        sx={(theme) => ({
          [theme.breakpoints.down("sm")]: {
            display: "none",
          },
        })}
      >
        <TopBarContent />
      </Box>
    </>
  );
}

export function TopBarContent(props: { vertical?: boolean; black?: boolean }) {
  const [user, team, profile, fetchUser] = useAuth(null);
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  return (
    <Box
      className={`${menu_bar}`}
      sx={(theme) => ({
        // small screen
        ...(props.vertical
          ? {
              flexDirection: "column",
              alignItems: "baseline",
            }
          : {}),
      })}
    >
      <BarItem
        tabIndex={1}
        command={() => {
          navigate("/");
        }}
        underlineColor={props.black ? "black" : "white"}
        label='HOME'
          selected={window.location.pathname === "/"}
      >
      </BarItem>
      {user && (
        <BarItem
          tabIndex={2}
          label="YOUR TEAM"
          selected={window.location.pathname === "/manage-team"}
          underlineColor={props.black ? "black" : "white"}
          command={() => {
            navigate("/manage-team");
          }}
        />
      )}
      <BarItem
        tabIndex={3}
        label="LEADERBOARD"
        selected={window.location.pathname === "/leaderboard"}
        underlineColor={props.black ? "black" : "white"}
        command={() => {
          navigate("/leaderboard");
        }}
      />

      <BarItem
        tabIndex={4}
        label="GAMES"
        selected={window.location.pathname === "/recent-games"}
        underlineColor={props.black ? "black" : "white"}
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
        underlineColor={props.black ? "black" : "white"}
        command={() => {
          window.open("https://docs.upac.dev/");
        }}
      />
      {user ? (
        <>
          <BarItem
            tabIndex={7}
            label="SIGN OUT"
            underlineColor={props.black ? "black" : "white"}
            command={() => {
              fetch(`${authUrl}/signout`).then(() => {
                fetchUser();
              });
            }}
          />
          <RawBarItem
            tabIndex={8}
            selected={window.location.pathname === "/profile"}
            underlineColor={props.black ? "black" : "white"}
            command={() => {
              navigate("/profile");
            }}
          >
            <Badge color="danger" invisible={profile !== null}>
              <Person />
            </Badge>
          </RawBarItem>
        </>
      ) : (
        <>
          <BarItem
            tabIndex={7}
            label="LOG IN"
            selected={window.location.pathname === "/login"}
            underlineColor={props.black ? "black" : "white"}
            command={() => {
              navigate("/login");
            }}
          />
          <BarItem
            tabIndex={8}
            label="SIGN UP"
            selected={window.location.pathname === "/signup"}
            underlineColor={props.black ? "black" : "white"}
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
        <BarItem label="© UPAC 2024" />
      </Box>
      <BarItem
        label="REPORT AN ISSUE"
        command={() => {
          window.open("https://github.com/alexwaeseperlman/Pokerbots/issues");
        }}
      />
    </Box>
  );
}
