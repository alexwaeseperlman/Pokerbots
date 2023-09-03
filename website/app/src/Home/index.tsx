import React from "react";
import Box from "@mui/joy/Box";
import Logo from "../components/Logo";
import { Button, Sheet, Typography } from "@mui/joy";
import { DiscordLogo } from "./Discord";
import Container from "@mui/joy/Container";
import graphic_small from "./graphic_small.png";
import graphic from "./graphic.png";
import BackgroundImage from "../components/BackgroundImage";

export default function HomePage() {
  return (
    <Container
      sx={{
        margin: "auto",
        pb: 16,
      }}
    >
      <BackgroundImage
        graphics={[`url(${graphic})`, `url(${graphic_small})`]}
      />
      <Box
        sx={{
          width: "100%",
          zIndex: 2,
          position: "relative",
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
            color="inherit"
            sx={{
              width: "100px",
              height: "100px",
            }}
          />
          <Typography textColor="inherit" level="h1" fontSize={64}>
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
            component={(props: any) => <a {...props} />}
            startDecorator={<DiscordLogo />}
            href="https://discord.gg/h4GNcUkAag"
          >
            Join our discord
          </Button>
        </Box>

        <Container
          maxWidth="sm"
          sx={{
            textAlign: "center",
            mt: 6,
          }}
        >
          <Typography textColor="inherit" level="body-md">
            The competition will start in 2024. For sponsorship inquiries,
            please contact alexwaeseperlman@gmail.com.
          </Typography>
        </Container>
      </Box>
    </Container>
  );
}
