import React from "react";
import Box from "@mui/system/Box";
import Logo from "../components/Logo";
import { FormControl, FormGroup, Input, TextField } from "@mui/material";
import { signup_input, signup_button } from "./styles.module.css";
import Container from "@mui/system/Container";

export default function HomePage() {
  return (
    <>
      <Box
        sx={{
          background: "linear-gradient(89.88deg, #CC385A 0%, #E76FBE 100%);",
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "space-around",
          color: "white",
        }}
      >
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
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
            <button className={signup_button}>Join now</button>
          </Box>
        </Box>
        <Box>
          <Container
            sx={{
              width: "600px",
            }}
          >
            The competition begins on November 1, 2023. For sponsorship
            inquiries, please contact pokerbotleague@mcgill.ca.
          </Container>
        </Box>
        <Box />
      </Box>
    </>
  );
}
