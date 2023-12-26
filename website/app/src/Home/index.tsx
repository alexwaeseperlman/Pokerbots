import React from "react";
import Box from "@mui/joy/Box";
import Logo from "../components/Logo";
import { Button, Link, Sheet, Typography } from "@mui/joy";
import { DiscordLogo } from "./Discord";
import Container from "@mui/joy/Container";
import graphic_small from "./graphic_small.png";
import graphic from "./graphic.png";
import BackgroundImage from "../components/BackgroundImage";
import HeaderFooter from "../components/HeaderFooter";

export function LogoText({ text }: { text: string }) {
  let vals1 = "0px 0px";
  for (let i = 0; i <= 20; i++) vals1 += `, ${i / 2}px ${i / 2}px`;
  let vals2 = "0px 0px";
  for (let i = 0; i <= 20; i++) vals2 += `, -${i / 2}px -${i / 2}px`;
  console.log(vals2);
  return (
    <Typography
      level="h1"
      color="inherit"
      sx={theme => ({
        fontSize: 100,
        [theme.breakpoints.down("sm")]: {
          fontSize: 88,
          letterSpacing: "5px",
        },
        position: "relative",
        letterSpacing: "10px",
        userSelect: "none",

        "&:before": {
          content: `"${text}"`,
          position: "absolute",
          zIndex: -1,
          top: 0,
          left: 0,
          textShadow: vals1,
          color: "#CDC0FF",
          mask: "repeating-linear-gradient(45deg, transparent 0 3px, rgba(0,0,0,0.5) 0 6px)",
          transition: "text-shadow 0.2s ease-out",
        },
        "&:hover:before": {
          textShadow: vals2
        },
      })}
    >
      {text}
    </Typography>
  );
}

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
          pb: 4,
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
            <LogoText text="UPAC" />
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
            please contact{" "}
            <Link href="mailto:alex@alexwp.com">alex@alexwp.com</Link>
          </Typography>
        </Box>
      </Container>
    </HeaderFooter>
  );
}
