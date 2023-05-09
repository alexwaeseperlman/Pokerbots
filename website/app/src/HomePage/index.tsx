import React from "react";
import Box from "@mui/system/Box";
import Logo from "../components/Logo";
import {
  Button,
  FormControl,
  FormGroup,
  Input,
  TextField,
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
  background: "#c8007c",
  display: "flex",
  opacity: 1,
}));

export default function HomePage() {
  return (
    <>
      <Box
        className={`${primary_background}`}
        sx={{
          width: "100%",
          minHeight: "90%",
          color: "white",
          zIndex: 1,
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
            backgroundSize: "contain",
            zIndex: -1,
            backgroundRepeat: "no-repeat",
            mixBlendMode: "screen",
            width: "100%",
            height: "100%",
            display: "block",
            overflow: "hidden",
            pointerEvents: "none",
          }}
        ></Box>
        <Box
          sx={{
            zIndex: 3,
            marginTop: "20px",
            width: "100%",
            height: "100%",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "space-around",
          }}
        >
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              marginTop: "50px",
            }}
          >
            <Logo
              sx={{
                width: "100px",
                height: "100px",
              }}
            />
            <h1>Poker Bot League</h1>
          </Box>
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
            }}
          >
            <Box
              sx={{
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
          </Box>
          <Box
            sx={{
              marginTop: "50px",
            }}
          >
            <Container
              sx={{
                width: "600px",
              }}
            >
              The competition begins on November 1, 2023. For sponsorship
              inquiries, please contact pokerbotleague@mcgill.ca. Lorem ipsum
            </Container>
          </Box>
          <Box />
        </Box>
      </Box>
    </>
  );
}
