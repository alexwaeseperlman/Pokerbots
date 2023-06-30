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
import { signup_input, signup_button } from "./styles.module.css";
import Container from "@mui/system/Container";
import bot_poker_image from "./bot_poker.png";
import styled from "@mui/system/styled";
import { primary_background } from "../styles.module.css";

const SignupButton = styled(Button)(({ theme }) => ({
  height: "40px",
  fontSize: "14px",
  padding: "0 10px",
  color: "white",
  width: "120px",
  justifyContent: "center",
  alignItems: "center",
  borderRadius: "0px 7px 7px 0px",
  outline: "none",
  border: "none",
  background: theme.palette.primary.main,
  ":hover": {
    background: theme.palette.primary.main,
  },
  display: "flex",
  opacity: 1,
  whiteSpace: "nowrap",
}));

export default function HomePage() {
  return (
    <>
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
          height: "100%",
        }}
      >
        <Box
          sx={{
            backgroundImage: `url(${bot_poker_image})`,
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
            <Typography variant="h2">Poker Bot League</Typography>
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
            <input
              placeholder="Join our mailing list"
              className={signup_input}
            ></input>
            <SignupButton>Join now</SignupButton>
          </Box>

          <Container
            sx={{
              maxWidth: "700px !important",
              textAlign: "center",
              mt: 6,
            }}
          >
            The competition will start in 2024. For sponsorship inquiries,
            please contact pokerbotleague@mcgill.ca.
          </Container>
        </Box>
      </Box>
    </>
  );
}
