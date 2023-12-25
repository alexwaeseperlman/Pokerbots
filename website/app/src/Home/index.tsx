import React from "react";
import Box from "@mui/joy/Box";
import Logo from "../components/Logo";
import { Button, Sheet, Typography } from "@mui/joy";
import { DiscordLogo } from "./Discord";
import Container from "@mui/joy/Container";
import graphic_small from "./graphic_small.png";
import graphic from "./graphic.png";
import BackgroundImage from "../components/BackgroundImage";
import HeaderFooter from "../components/HeaderFooter";

export default function HomePage() {
  return (
    <HeaderFooter>
      <Container
        sx={{
          margin: "auto",
          gridArea: "content",
          height: "100%",
          flexDirection: "column",
          justifyContent: "center",
          display: "flex",
          pb: 4
        }}
      >
        <Box
          sx={{
            flexDirection: "column",
            justifyContent: "center",
            display: "flex",
            maxWidth: "700px",
            flexGrow: 1,
            gap: 4,
          }}
        >
          <Box
            sx={{
              display: "flex",
              flexDirection: "column",
              alignItems: "left",
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
              display: "flex",
              flexDirection: "row",
              alignItems: "left",
              justifyContent: "left",
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
          <Typography textColor="inherit" level="body-md">
            The competition will start in 2024. For sponsorship inquiries,
            please contact alexwaeseperlman@gmail.com.
          </Typography>
        </Box>
      </Container>
    </HeaderFooter>
  );
}
