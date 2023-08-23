import { CssVarsThemeOptions } from "@mui/joy";
import React from "react";

declare module "@mui/material/styles" {
  interface TypeBackground {
    active: string;
    gradient: string;
  }
}

const theme: CssVarsThemeOptions = {
  colorSchemes: {
    light: {
      palette: {
        mode: "light",

        primary: {
          mainChannel: "#CD3939",
        },

        background: {
          body: "linear-gradient(269.89deg, #392889 0%, #191335 100%)",
          level1: "white",
          level2: "#ffffff",
          popup: "#f5f5f5",
          surface: "black",
        },

        text: {
          primary: "white",
          secondary: "white",
          tertiary: "white",
        },
      },
    },
  },
  fontFamily: {
    body: "Figtree",
    code: "Fira Code",
    display: "Figtree",
  },
  typography: {
    h1: {
      fontWeight: 700,
    },

    h2: {
      fontWeight: 700,
    },
  },
};

export default theme;
