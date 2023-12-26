import { Box, Typography } from "@mui/material";
import graphic from "./404.webp";
import React from "react";
import BackgroundImage from "../components/BackgroundImage";
import HeaderFooter from "../components/HeaderFooter";
import { Link } from "@mui/joy";

export default function ErrorPage() {
  return (
    <HeaderFooter graphics={[`url(${graphic})`]}>
      <Box
        sx={{
          gridArea: "content",
        }}
      >
        <Box
          sx={{
            zIndex: 1,
          }}
        >
          <Typography variant="h2" color="inherit">
            Something went wrong. Try reloading the page.
          </Typography>
          <Typography color="inherit">
            If the problem persists, please contact us at{" "}
            <Link href="mailto:alex@alexwp.com">alex@alexwp.com</Link>
          </Typography>
        </Box>
      </Box>
    </HeaderFooter>
  );
}
