import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { SettingsApp } from "./SettingsApp";

function mount() {
  const root = document.getElementById("root");
  if (!root) {
    throw new Error("missing #root element");
  }
  const role = window.__promptlyWindowRole ?? "main";
  createRoot(root).render(
    <StrictMode>
      {role === "settings" ? <SettingsApp /> : <App />}
    </StrictMode>,
  );
}

// vite-plugin-singlefile inlines the bundle in <head>, before <body> is parsed.
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", mount);
} else {
  mount();
}
