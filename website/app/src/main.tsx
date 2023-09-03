import * as React from "react";
import * as ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { SnackbarProvider } from "notistack";

//import "../static/css/styles.css";
import UPAC from "./UPAC";
import { ThemeProvider, CssVarsProvider, CssBaseline } from "@mui/joy";
import theme from "./theme";

function RootApp() {
  return (
    <React.StrictMode>
      <CssVarsProvider theme={theme}>
        <SnackbarProvider
          maxSnack={3}
          SnackbarProps={{
            style: {
              fontFamily: "Figtree",
            },
          }}
        >
          <BrowserRouter>
            <UPAC />
          </BrowserRouter>
        </SnackbarProvider>
      </CssVarsProvider>
    </React.StrictMode>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RootApp />
);
