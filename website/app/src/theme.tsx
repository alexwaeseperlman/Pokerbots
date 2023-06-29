import {
  createTheme,
  PaletteColorOptions,
  ThemeOptions,
  TypeBackground,
} from "@mui/material";
import React from "react";

declare module "@mui/material/styles" {
  interface TypeBackground {
    active: string;
    gradient: string;
  }
}

export default createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#CD3939",
    },
    secondary: {
      main: "#281f5f",
      contrastText: "#000",
    },
    background: {
      default: "#e0e0e0",
      paper: "#ffffff",
      active: "#f5f5f5",
    },
  },
  typography: {
    fontFamily: "Figtree",
    h1: {
      fontWeight: 700,
    },

    h2: {
      fontWeight: 700,
    },
  },
  shape: {
    borderRadius: 2,
  },
});
