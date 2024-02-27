import React from "react";
import Box from "@mui/joy/Box";
import Logo from "../components/Logo";
import { Button, Divider, Link, Sheet, Typography, useTheme } from "@mui/joy";
import { DiscordLogo } from "./Discord";
import Container from "@mui/joy/Container";
import graphic_small from "./graphic_small.png";
import graphic from "./graphic.png";
import BackgroundImage from "../components/BackgroundImage";
import HeaderFooter from "../components/HeaderFooter";
import Logos from "./Sponsors/Sponsors";
import FAQ from "./FAQ";

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
      sx={(theme) => ({
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
          transition: "text-shadow 0.2s ease",
        },
        "&:hover:before": {
          textShadow: vals2,
        },
      })}
    >
      {text}
    </Typography>
  );
}

function InfoSection(props: { title: string; children: React.ReactNode }) {
  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
      <Typography level="h3" color="inherit">
        {props.title}
      </Typography>
      {props.children}
    </Box>
  );
}

function InfoSections() {
  return (
    <Box
      sx={(theme) => ({
        display: "flex",
        flexDirection: "row",
        gap: 2,
        justifyContent: "space-between",
        [theme.breakpoints.down("sm")]: {
          flexDirection: "column",
          gap: 4,
        },
      })}
    >
      <InfoSection title="Design a bot">
        <Typography level="body-md" color="inherit">
          Develop a poker algorithm in a programming language of your choice
        </Typography>
        <Link href="https://docs.upac.dev/">View documentation</Link>
      </InfoSection>
      <InfoSection title="Compete">
        <Typography level="body-md" color="inherit">
          Your algorithms will automatically be tested against each other on our
          platform
        </Typography>
        <Link href="/recent-games">View recent games</Link>
      </InfoSection>
      <InfoSection title="Get ranked">
        <Typography level="body-md" color="inherit">
          We will calculate your Elo, and rank you against other competitors
        </Typography>
        <Link href="/leaderboard">View leaderboard</Link>
      </InfoSection>
    </Box>
  );
}

export default function HomePage() {
  return (
    <HeaderFooter>
      <Box
        sx={{
          mt: 16,
          gridArea: "content / content / content / extra",
          flexDirection: "column",
          justifyContent: "center",
          display: "flex",
          gap: 10,
        }}
      >
        <Box
          sx={{
            display: "flex",
            flexDirection: "column",
            alignItems: "left",
            gap: 2,
          }}
        >
          <LogoText text="UPAC" />
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
        </Box>
        <Box>
          <Typography level="h3" color="inherit">
            Sponsored by
          </Typography>
          <Box
            sx={{
              display: "flex",
              flexDirection: "row",
              alignItems: "center",
              gap: 4,
            }}
          >
            <Logos />
          </Box>
          <Typography textColor="inherit" level="body-md" mt={1}>
            For sponsorship inquiries, please contact{" "}
            <Link href="mailto:alex@alexwp.com">alex@alexwp.com</Link>
          </Typography>
        </Box>
        <InfoSections />
        <FAQ />
      </Box>
    </HeaderFooter>
  );
}
