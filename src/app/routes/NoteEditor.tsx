import { useEffect, useState, useCallback, useRef } from "react"
import { useEditor, EditorContent } from "@tiptap/react"
import StarterKit from "@tiptap/starter-kit"
import { invoke } from "@tauri-apps/api/core"
import { useNotes } from "../providers/NotesProvider"
import type { Note } from "../../lib/types"

export function NoteEditor() {
  const { selectedNote, refreshNotes } = useNotes()
  const [saveStatus, setSaveStatus] = useState<"saved" | "saving" | "unsaved">("saved")
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const noteIdRef = useRef<string | null>(null)

  const editor = useEditor({
    extensions: [StarterKit],
    content: "",
    onUpdate: () => {
      setSaveStatus("unsaved")
      // Auto-save after 2 seconds of inactivity
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current)
      }
      saveTimerRef.current = setTimeout(() => {
        handleSave()
      }, 2000)
    },
  })

  // Load note content when selection changes
  useEffect(() => {
    if (selectedNote && editor) {
      noteIdRef.current = selectedNote.id
      editor.commands.setContent(markdownToHtml(selectedNote.bodyMarkdown))
      setSaveStatus("saved")
    }
  }, [selectedNote?.id, editor])

  const handleSave = useCallback(async () => {
    if (!editor || !noteIdRef.current) return
    setSaveStatus("saving")
    try {
      const markdown = htmlToMarkdown(editor.getHTML())
      await invoke<Note>("save_note", {
        noteId: noteIdRef.current,
        markdown,
      })
      setSaveStatus("saved")
      await refreshNotes()
    } catch (e) {
      console.error("Save failed:", e)
      setSaveStatus("unsaved")
    }
  }, [editor, refreshNotes])

  // Ctrl+S handler
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault()
        if (saveTimerRef.current) {
          clearTimeout(saveTimerRef.current)
        }
        handleSave()
      }
    }
    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [handleSave])

  if (!selectedNote) {
    return (
      <div style={{ padding: "16px", color: "#888" }}>
        Select a note from the sidebar to start editing.
      </div>
    )
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "8px 0",
        borderBottom: "1px solid #e0e0e0",
        marginBottom: "12px",
      }}>
        <h2 style={{ margin: 0, fontSize: "18px" }}>{selectedNote.title}</h2>
        <span style={{
          fontSize: "12px",
          color: saveStatus === "saved" ? "#4caf50" : saveStatus === "saving" ? "#ff9800" : "#f44336",
        }}>
          {saveStatus === "saved" ? "Saved" : saveStatus === "saving" ? "Saving..." : "Unsaved"}
        </span>
      </div>
      <div style={{ flex: 1, overflow: "auto" }}>
        <EditorContent editor={editor} />
      </div>
    </div>
  )
}

function markdownToHtml(markdown: string): string {
  // Simple markdown to HTML conversion for Tiptap
  let html = markdown
    .split("\n\n")
    .map((block) => {
      const trimmed = block.trim()
      if (!trimmed) return ""
      if (trimmed.startsWith("# ")) return `<h1>${trimmed.slice(2)}</h1>`
      if (trimmed.startsWith("## ")) return `<h2>${trimmed.slice(3)}</h2>`
      if (trimmed.startsWith("### ")) return `<h3>${trimmed.slice(4)}</h3>`
      if (trimmed.startsWith("- ")) {
        const items = trimmed.split("\n").map((line) => {
          const content = line.replace(/^- /, "")
          return `<li>${content}</li>`
        })
        return `<ul>${items.join("")}</ul>`
      }
      if (trimmed.startsWith("```")) {
        const lines = trimmed.split("\n")
        const code = lines.slice(1, -1).join("\n")
        return `<pre><code>${code}</code></pre>`
      }
      return `<p>${trimmed.replace(/\n/g, "<br>")}</p>`
    })
    .join("")
  return html
}

function htmlToMarkdown(html: string): string {
  // Simple HTML to Markdown conversion
  let md = html
  md = md.replace(/<h1>(.*?)<\/h1>/g, "# $1\n\n")
  md = md.replace(/<h2>(.*?)<\/h2>/g, "## $1\n\n")
  md = md.replace(/<h3>(.*?)<\/h3>/g, "### $1\n\n")
  md = md.replace(/<ul>(.*?)<\/ul>/gs, (_, content: string) => {
    return content.replace(/<li>(.*?)<\/li>/g, "- $1\n") + "\n"
  })
  md = md.replace(/<pre><code>(.*?)<\/code><\/pre>/gs, "```\n$1\n```\n\n")
  md = md.replace(/<p>(.*?)<\/p>/gs, (_, content: string) => {
    return content.replace(/<br\s*\/?>/g, "\n") + "\n\n"
  })
  md = md.replace(/<strong>(.*?)<\/strong>/g, "**$1**")
  md = md.replace(/<em>(.*?)<\/em>/g, "*$1*")
  md = md.replace(/<code>(.*?)<\/code>/g, "`$1`")
  md = md.replace(/<[^>]+>/g, "")
  md = md.replace(/\n{3,}/g, "\n\n")
  return md.trim() + "\n"
}
