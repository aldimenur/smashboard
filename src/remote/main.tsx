import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { RemoteApp } from "./RemoteApp";
import "./remote.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RemoteApp />
  </StrictMode>,
);
