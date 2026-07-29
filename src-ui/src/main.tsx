import React, { useState, useEffect } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";
import { loadLanguage } from "./lib/utils";

function AppWrapper() {
  const [languageLoaded, setLanguageLoaded] = useState(false);

  useEffect(() => {
    loadLanguage().then(() => {
      setLanguageLoaded(true);
    });
  }, []);

  if (!languageLoaded) {
    return null;
  }

  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <AppWrapper />
  </React.StrictMode>
);