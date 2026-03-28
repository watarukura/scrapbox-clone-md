import React from "react";
import ReactDOM from "react-dom/client";
import "./styles/global.css";
import { WorkspaceProvider } from "./app/providers/WorkspaceProvider";
import { NotesProvider } from "./app/providers/NotesProvider";
import { AppShell } from "./app/layout/AppShell";
import { Home } from "./app/routes/Home";
import { NoteEditor } from "./app/routes/NoteEditor";
import { useNotes } from "./app/providers/NotesProvider";

function MainContent() {
  const { selectedNote } = useNotes();
  return selectedNote ? <NoteEditor /> : <Home />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <WorkspaceProvider>
      <NotesProvider>
        <AppShell>
          <MainContent />
        </AppShell>
      </NotesProvider>
    </WorkspaceProvider>
  </React.StrictMode>,
);
