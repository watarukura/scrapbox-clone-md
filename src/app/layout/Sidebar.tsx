import { useState } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useWorkspace } from "../providers/WorkspaceProvider"
import { useNotes } from "../providers/NotesProvider"
import type { Note } from "../../lib/types"

export function Sidebar() {
  const { workspace } = useWorkspace()
  const { notes, selectedNote, selectNote, refreshNotes, workspaceId } = useNotes()
  const [searchQuery, setSearchQuery] = useState("")
  const [searchResults, setSearchResults] = useState<Note[] | null>(null)

  const handleSearch = async () => {
    if (!workspaceId || !searchQuery.trim()) {
      setSearchResults(null)
      return
    }
    const results = await invoke<Note[]>("search_notes", {
      workspaceId,
      query: searchQuery,
    })
    setSearchResults(results)
  }

  const handleClearSearch = () => {
    setSearchQuery("")
    setSearchResults(null)
  }

  const handleCreateNote = async () => {
    if (!workspace) return
    const fileName = `new-note-${Date.now()}.md`
    const content = `# New Note\n\n`
    await invoke("save_new_file", {
      rootPath: workspace.rootPath,
      fileName,
      content,
    }).catch(() => {
      // save_new_file not implemented yet, ignore
    })
    await refreshNotes()
  }

  const displayNotes = searchResults ?? notes

  return (
    <aside style={{
      width: "260px",
      borderRight: "1px solid #e0e0e0",
      display: "flex",
      flexDirection: "column",
      backgroundColor: "#f5f5f5",
      overflow: "hidden",
    }}>
      {workspace && (
        <div style={{ padding: "12px", borderBottom: "1px solid #e0e0e0" }}>
          <div style={{ fontWeight: "bold", fontSize: "14px", marginBottom: "8px" }}>
            {workspace.name}
          </div>
          <div style={{ display: "flex", gap: "4px" }}>
            <input
              type="text"
              placeholder="Search..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && handleSearch()}
              style={{
                flex: 1,
                padding: "4px 8px",
                border: "1px solid #ccc",
                borderRadius: "4px",
                fontSize: "13px",
              }}
            />
            {searchResults && (
              <button onClick={handleClearSearch} style={{ fontSize: "12px", cursor: "pointer" }}>
                ✕
              </button>
            )}
          </div>
        </div>
      )}
      <div style={{ flex: 1, overflowY: "auto" }}>
        {displayNotes.map((note) => (
          <div
            key={note.id}
            onClick={() => selectNote(note.id)}
            style={{
              padding: "8px 12px",
              cursor: "pointer",
              backgroundColor: selectedNote?.id === note.id ? "#e0e0e0" : "transparent",
              borderBottom: "1px solid #eee",
              fontSize: "13px",
            }}
          >
            {note.title}
          </div>
        ))}
      </div>
      {workspace && (
        <div style={{ padding: "8px", borderTop: "1px solid #e0e0e0" }}>
          <button
            onClick={handleCreateNote}
            style={{
              width: "100%",
              padding: "6px",
              cursor: "pointer",
              fontSize: "13px",
            }}
          >
            + New Note
          </button>
        </div>
      )}
    </aside>
  )
}
