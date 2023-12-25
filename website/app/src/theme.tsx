import { CssVarsThemeOptions, extendTheme } from "@mui/joy";
import React from "react";
import "./styles.module.css";
import { ColorSystemOptions } from "@mui/joy/styles/extendTheme";

const light: ColorSystemOptions = {
  palette: {
    mode: "light",

    primary: {
      //plainHoverBg: "#0a6bcb11",
      //plainActiveBg: "#0a6bcb22",
      //plainColor: "#96c1eb",
    },
  },
};
const theme = extendTheme({
  colorSchemes: {
    light,
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
