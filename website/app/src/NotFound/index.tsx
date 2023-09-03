import { Box, Typography } from "@mui/material";
import graphic from "./graphic.png";
import React from "react";
import BackgroundImage from "../components/BackgroundImage";

export default function NotFound() {
  return (
    <Box
      sx={{
        width: "100%",
        p: 4,
        pb: 16,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        flexGrow: 1,
      }}
    >
      <BackgroundImage graphics={[`url(${graphic})`]} />
      <Box
        sx={{
          zIndex: 1,
        }}
      >
        <Typography variant="h2" color="inherit">
          There is no page at this address.
        </Typography>
      </Box>
    </Box>
  );
}
