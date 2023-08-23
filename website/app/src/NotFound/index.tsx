import { Box, Typography } from "@mui/material";
import graphic from "./graphic.png";
import React from "react";

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
      <Box
        sx={{
          backgroundImage: `url(${graphic})`,
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
          zIndex: 1,
        }}
      >
        <Typography variant="h2">There is no page at this address.</Typography>
      </Box>
    </Box>
  );
}
