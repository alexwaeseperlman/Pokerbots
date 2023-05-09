import * as React from "react";
import * as ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import "./styles.module.css";

//import "../static/css/styles.css";
import PokerZero from "./PokerZero";

function RootApp() {
  return (
    <BrowserRouter>
      <PokerZero />
    </BrowserRouter>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <RootApp />
);
