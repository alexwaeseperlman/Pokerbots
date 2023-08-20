import React from "react";
import Box from "@mui/system/Box";
import Logo from "../components/Logo";
import {
  Button,
  FormControl,
  FormGroup,
  Input,
  TextField,
  Typography,
} from "@mui/material";
import { DiscordLogo } from "./Discord";
import { signup_input, signup_button } from "./styles.module.css";
import Container from "@mui/system/Container";
import graphic from "./graphic.png";
import graphic_small from "./graphic_small.png";
import styled from "@mui/system/styled";
import { primary_background } from "../styles.module.css";

export default function HomePage() {
  return (
    <Box
      className={`${primary_background}`}
      sx={{
        width: "100%",
        color: "white",
        p: 4,
        pb: 16,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        flexGrow: 1,
      }}
    >
      <Box
        sx={{
          backgroundImage: `url(${graphic_small})`,
          filter: "grayscale(100%)",
          opacity: 0.4,
          backgroundPosition: "center",
          position: "absolute",
          top: 0,
          left: 0,
          backgroundSize: "contain",
          backgroundRepeat: "no-repeat",
          mixBlendMode: "screen",
          width: "100%",
          maxWidth: "100vw",
          height: "100%",
          display: "block",
          overflow: "hidden",
          pointerEvents: "none",
        }}
      ></Box>
      <Box
        sx={{
          width: "100%",
        }}
      >
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            textAlign: "center",
          }}
        >
          <Logo
            sx={{
              width: "100px",
              height: "100px",
            }}
          />
          <Typography variant="h2">UPAC</Typography>
        </Box>
        <Box
          sx={{
            mt: 6,
            display: "flex",
            flexDirection: "row",
            alignItems: "center",
            justifyContent: "center",
            width: "100%",
          }}
        >
          <Button
            variant="contained"
            startIcon={<DiscordLogo />}
            href="https://discord.gg/h4GNcUkAag"
          >
            Join our discord
          </Button>
        </Box>

        <Container
          sx={{
            maxWidth: "700px !important",
            textAlign: "center",
            mt: 6,
          }}
        >
          The competition will start in 2024. For sponsorship inquiries, please
          contact alexwaeseperlman@gmail.com.
        </Container>
      </Box>
    </Box>
  );
}
