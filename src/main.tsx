import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { LocaleProvider } from "./lib/i18n";
import { StoreProvider } from "./lib/store";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <LocaleProvider>
      <StoreProvider>
        <App />
      </StoreProvider>
    </LocaleProvider>
  </React.StrictMode>,
);
