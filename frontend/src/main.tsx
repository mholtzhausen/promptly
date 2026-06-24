import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";

function mount() {
  const root = document.getElementById("root");
  if (!root) {
    throw new Error("missing #root element");
  }
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

// vite-plugin-singlefile inlines the bundle in <head>, before <body> is parsed.
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", mount);
} else {
  mount();
}