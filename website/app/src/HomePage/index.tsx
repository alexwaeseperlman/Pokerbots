import React from "react";
import Box from "@mui/joy/Box";
import Logo from "../components/Logo";
import { Button, Sheet, Typography } from "@mui/joy";
import { DiscordLogo } from "./Discord";
import Container from "@mui/joy/Container";
import graphic_small from "./graphic_small.png";

export default function HomePage() {
  return (
    <Container
      sx={{
        margin: "auto",
        pb: 16,
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
          <Typography level="h2" textColor="white">
            UPAC
          </Typography>
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
            variant="solid"
            as="a"
            startDecorator={<DiscordLogo />}
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
          <Typography level="body-md" textColor="white">
            The competition will start in 2024. For sponsorship inquiries,
            please contact alexwaeseperlman@gmail.com.
          </Typography>
        </Container>
      </Box>
    </Container>
  );
}
