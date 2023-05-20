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
      main: "#C8BDFE",
      contrastText: "#000",
    },
    background: {
      default: "#e0e0e0",
      paper: "#ffffff",
      active: "#f5f5f5",
    },
  },
  shape: {
    borderRadius: 2,
  },
});
