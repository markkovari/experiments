import { useEffect, useRef, useState } from 'react'
import { EditorView, keymap, lineNumbers, highlightActiveLine, drawSelection } from '@codemirror/view'
import { EditorState, Compartment } from '@codemirror/state'
import { yaml } from '@codemirror/lang-yaml'
import { oneDark } from '@codemirror/theme-one-dark'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { vim, Vim } from '@replit/codemirror-vim'
import {
  syntaxHighlighting,
  defaultHighlightStyle,
  bracketMatching,
} from '@codemirror/language'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'

// Tell Vim's ex-command ':w' to trigger our save callback
let globalSaveCallback: (() => void) | null = null
Vim.defineEx('w', '', () => { globalSaveCallback?.() })
Vim.defineEx('wq', '', () => { globalSaveCallback?.() })

interface Props {
  value: string
  onChange: (value: string) => void
  onSave?: () => void
  vimMode?: boolean
}

export function YamlEditor({ value, onChange, onSave, vimMode = true }: Props) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewRef = useRef<EditorView | null>(null)
  const onChangeRef = useRef(onChange)
  const onSaveRef = useRef(onSave)
  const vimCompartment = useRef(new Compartment())
  const [mode, setMode] = useState<string>('NORMAL')

  // Keep callback refs fresh without recreating the editor
  useEffect(() => { onChangeRef.current = onChange }, [onChange])
  useEffect(() => { onSaveRef.current = onSave }, [onSave])
  useEffect(() => { globalSaveCallback = onSave ?? null }, [onSave])

  // Build editor once
  useEffect(() => {
    if (!containerRef.current) return

    const updateListener = EditorView.updateListener.of(update => {
      if (update.docChanged) {
        onChangeRef.current(update.state.doc.toString())
      }
    })

    // Vim mode indicator: poll the statusbar element the vim extension attaches
    const vimStatusListener = EditorView.updateListener.of(update => {
      const statusbar = (update.state as unknown as { statusbar?: Element }).statusbar
      if (statusbar) {
        const text = statusbar.textContent?.trim() ?? ''
        setMode(
          text.includes('INSERT') ? 'INSERT' :
          text.includes('VISUAL') ? 'VISUAL' :
          'NORMAL'
        )
      }
    })

    const state = EditorState.create({
      doc: value,
      extensions: [
        vimCompartment.current.of(vimMode ? vim({ status: true }) : []),
        yaml(),
        oneDark,
        lineNumbers(),
        highlightActiveLine(),
        drawSelection(),
        bracketMatching(),
        highlightSelectionMatches(),
        history(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          ...searchKeymap,
          { key: 'Ctrl-s', run: () => { onSaveRef.current?.(); return true } },
        ]),
        updateListener,
        vimStatusListener,
        EditorView.theme({
          '&': { height: '100%', fontSize: '12px' },
          '.cm-scroller': { fontFamily: 'ui-monospace, monospace', overflow: 'auto' },
          '.cm-vim-statusbar': {
            fontFamily: 'ui-monospace, monospace',
            fontSize: '11px',
            padding: '2px 8px',
            borderTop: '1px solid #333',
            color: '#aaa',
            background: '#1a1a1a',
          },
        }),
      ],
    })

    const view = new EditorView({ state, parent: containerRef.current })
    viewRef.current = view

    return () => {
      view.destroy()
      viewRef.current = null
      globalSaveCallback = null
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Sync value from outside without disturbing cursor/selection
  useEffect(() => {
    const view = viewRef.current
    if (!view) return
    const current = view.state.doc.toString()
    if (current === value) return
    view.dispatch({
      changes: { from: 0, to: current.length, insert: value },
    })
  }, [value])

  // Toggle vim mode dynamically
  useEffect(() => {
    const view = viewRef.current
    if (!view) return
    view.dispatch({
      effects: vimCompartment.current.reconfigure(vimMode ? vim({ status: true }) : []),
    })
  }, [vimMode])

  return (
    <div className="flex flex-col h-full">
      {/* mode indicator */}
      <div className="flex items-center gap-2 px-3 py-1 bg-gray-900 border-b border-gray-700 shrink-0 text-xs font-mono">
        <span className={`px-1.5 py-0.5 rounded text-[10px] font-bold tracking-wider ${
          mode.includes('INSERT') ? 'bg-green-700 text-green-100' :
          mode.includes('VISUAL') ? 'bg-purple-700 text-purple-100' :
          'bg-blue-800 text-blue-100'
        }`}>
          {mode.includes('INSERT') ? 'INSERT' : mode.includes('VISUAL') ? 'VISUAL' : 'NORMAL'}
        </span>
        <span className="text-gray-500">vim · yaml</span>
        <span className="ml-auto text-gray-600 text-[10px]">:w to save · Ctrl-S to save</span>
      </div>
      <div ref={containerRef} className="flex-1 overflow-hidden" />
    </div>
  )
}
