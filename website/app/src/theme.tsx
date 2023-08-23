import { CssVarsThemeOptions, extendTheme } from "@mui/joy";
import React from "react";
import "./styles.module.css";
import { ColorSystemOptions } from "@mui/joy/styles/extendTheme";

const light: ColorSystemOptions = {
  palette: {
    mode: "light",
    background: {
      body: "white",
      level1: "white",
      level2: "#ffffff",
      popup: "#f5f5f5",
      surface: "white",
      backdrop: "white",
      level3: "#f5f5f5",
    },

    text: {
      primary: "black",
      secondary: "black",
      tertiary: "black",
    },

    neutral: {
      solidBg: "#392889",
      solidColor: "white",
    },
  },
};
const theme = extendTheme({
  colorSchemes: {
    light,
    dark: light,
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
});

export default theme;
