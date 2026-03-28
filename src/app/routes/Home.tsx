import { useEffect } from "react"
import { open } from "@tauri-apps/plugin-dialog"
import { useWorkspace } from "../providers/WorkspaceProvider"
import { useNotes } from "../providers/NotesProvider"

export function Home() {
  const { workspace, openWorkspace } = useWorkspace()
  const { loadNotes } = useNotes()

  const handleOpenFolder = async () => {
    const selected = await open({ directory: true, multiple: false })
    if (selected) {
      await openWorkspace(selected)
    }
  }

  useEffect(() => {
    if (workspace) {
      loadNotes(workspace.id)
    }
  }, [workspace, loadNotes])

  if (!workspace) {
    return (
      <div style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        height: "100%",
        gap: "16px",
      }}>
        <h2>Welcome to Scrapbox Clone MD</h2>
        <p>Select a folder to open as workspace</p>
        <button
          onClick={handleOpenFolder}
          style={{
            padding: "10px 24px",
            fontSize: "16px",
            cursor: "pointer",
          }}
        >
          Open Folder
        </button>
      </div>
    )
  }

  return (
    <div style={{ padding: "16px" }}>
      <p>Select a note from the sidebar to start editing.</p>
    </div>
  )
}
