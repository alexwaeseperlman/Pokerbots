import { Box, Typography } from "@mui/material";
import graphic from "./404.webp";
import React from "react";
import BackgroundImage from "../components/BackgroundImage";
import HeaderFooter from "../components/HeaderFooter";

export default function NotFound() {
  return (
    <HeaderFooter graphics={[`url(${graphic})`]}>
      <Box
        sx={{
          gridArea:'content',
        }}
      >
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
    </HeaderFooter>
  );
}
