import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { ArrowRight, CheckCircle2, ChevronDown, CircleHelp, File, FileArchive, Files, FolderOpen, Layers3, LockKeyhole, MousePointer2, Play, Plus, RotateCcw, ShieldCheck, Sparkles, X } from "lucide-react";

type Recipe = { id: string; name: string; description: string; accent: string; detail: string };
type PlanItem = { source: string; destination: string; bytes: number };
type Preview = { plan_id: string; output_root: string; recipe_id: string; items: PlanItem[]; warnings: string[]; total_bytes: number };
type RunResult = { completed: string[]; failed: string[] };

const recipeArt: Record<string, typeof Layers3> = {
  organize: Layers3,
  rename: Files,
  collect: FileArchive,
};

const RECIPES: Recipe[] = [
  { id: "organize", name: "Organize by type", description: "Sort copies into folders by file type.", accent: "peach", detail: "Images · Documents · Media · Archives · Other" },
  { id: "rename", name: "Rename cleanly", description: "Create sequentially numbered copies.", accent: "mint", detail: "001-file · 002-file · 003-file" },
  { id: "collect", name: "Collect for handoff", description: "Gather files in a single delivery folder.", accent: "lavender", detail: "Copies only · Originals untouched" },
];

function basename(path: string) { return path.split(/[\\/]/).pop() || path; }
function formatBytes(n: number) { if (n < 1024) return `${n} B`; if (n < 1048576) return `${(n / 1024).toFixed(1)} KB`; return `${(n / 1048576).toFixed(1)} MB`; }
function errorText(e: unknown) { return e instanceof Error ? e.message : String(e); }
function displayPath(path: string) {
  if (path.startsWith("\\\\?\\UNC\\")) return "\\\\" + path.slice(8);
  if (path.startsWith("\\\\?\\")) return path.slice(4);
  return path;
}
function executionErrorText(e: unknown) {
  const message = errorText(e).trim();
  return /preview again\.?$/i.test(message) ? message : `${message} Preview again.`;
}

export default function App() {
  const [paths, setPaths] = useState<string[]>([]);
  const [destination, setDestination] = useState<string>("");
  const [recipe, setRecipe] = useState("organize");
  const [preview, setPreview] = useState<Preview | null>(null);
  const [result, setResult] = useState<RunResult | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [visibleCount, setVisibleCount] = useState(100);
  const [stage, setStage] = useState<"select" | "preview" | "done">("select");
  const revision = useRef(0);
  const busyRef = useRef(false);
  const invalidate = () => { revision.current += 1; setPreview(null); setResult(null); setVisibleCount(100); setStage("select"); };
  const recipeInfo = useMemo(() => RECIPES.find((r) => r.id === recipe)!, [recipe]);

  const addPaths = useCallback((newPaths: string[]) => {
    if (busyRef.current) { setError("Wait for the current operation to finish before changing files."); return; }
    setPaths((old) => [...new Set([...old, ...newPaths])]);
    revision.current += 1; setPreview(null); setResult(null); setError(""); setStage("select");
  }, []);

  useEffect(() => {
    let off: undefined | (() => void);
    getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") setDragging(true);
      if (event.payload.type === "leave") setDragging(false);
      if (event.payload.type === "drop") {
        setDragging(false);
        addPaths(event.payload.paths);
      }
    }).then((unsubscribe) => { off = unsubscribe; }).catch((e) => setError(`Drop handler unavailable: ${errorText(e)}`));
    return () => { off?.(); };
  }, [addPaths]);

  async function chooseFiles() {
    try {
      const selected = await open({ directory: false, multiple: true, title: "Choose files" });
      if (Array.isArray(selected)) addPaths(selected);
      else if (typeof selected === "string") addPaths([selected]);
    } catch (e) { setError(errorText(e)); }
  }
  async function chooseFolder() {
    try {
      const chosen = await open({ directory: true, multiple: false, title: "Choose output folder" });
      if (typeof chosen === "string") { setDestination(chosen); invalidate(); }
    } catch (e) { setError(errorText(e)); }
  }
  async function previewWork() {
    setError(""); setResult(null); busyRef.current = true; setBusy(true);
    const requestedRevision = revision.current;
    try {
      const p = await invoke<Preview>("preview_recipe", { paths, recipeId: recipe, outputDirectory: destination });
      if (requestedRevision === revision.current) { setPreview(p); setVisibleCount(100); setStage("preview"); }
    } catch (e) { setError(errorText(e)); }
    finally { busyRef.current = false; setBusy(false); }
  }
  async function runWork() {
    if (!preview) return;
    setError(""); busyRef.current = true; setBusy(true);
    const requestedRevision = revision.current;
    try {
      const r = await invoke<RunResult>("execute_recipe", { planId: preview.plan_id });
      if (requestedRevision === revision.current) { setResult(r); setStage("done"); }
    } catch (e) { setPreview(null); setStage("select"); setError(executionErrorText(e)); }
    finally { busyRef.current = false; setBusy(false); }
  }
  function removeFile(path: string) { setPaths((p) => p.filter((f) => f !== path)); invalidate(); }
  function reset() { setPaths([]); setDestination(""); invalidate(); setError(""); }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand"><span className="brand-mark"><Sparkles size={21}/></span><span className="brand-text">intent<span className="brand-dot">.</span><small>THE DESKTOP ACTION LAYER</small></span></div>
        <div className="nav-label">WORKSPACE</div>
        <div className="nav-item active"><Layers3 size={17} /> Workspace <span className="nav-pill">01</span></div>
        <div className="sidebar-bottom">
          <div className="sidebar-note"><Sparkles size={17}/><div><strong>Built for two worlds.</strong><span>One workflow engine. Windows and macOS.</span></div></div>
          <div className="sidebar-footer"><ShieldCheck size={15}/> Local-first · Open source</div>
        </div>
      </aside>
      <main className="main">
        <div className="topbar"><div className="breadcrumbs">Workspace <span>/</span> File actions</div><div className="top-right"><span className="status-dot"/> Private by design <span className="beta">ALPHA 0.1</span></div></div>
        <div className="content">
          <div className="eyebrow"><span className="eyebrow-line"/> WORK WITH INTENT</div>
          <div className="page-title"><div><h1>Do more with less.</h1><p>Choose the outcome. We handle the steps.</p></div><button className="reset-button" onClick={reset} disabled={busy}><RotateCcw size={15}/> Start over</button></div>
          <div className="step-header"><span className="step-number">01</span><h2>Drop your files</h2><span className="step-sub">Your files stay on this device</span></div>
          <button className={`dropzone ${dragging ? "dragging" : ""}`} onClick={chooseFiles} disabled={busy} aria-label="Choose files or drop them here">
            <span className="drop-icon"><Plus size={26}/></span>
            <strong>{paths.length ? `${paths.length} file${paths.length === 1 ? "" : "s"} selected` : "Bring your files here"}</strong>
            <span>Drag files into this window, or <span className="inline-link">browse files</span></span>
            <small>Documents, images, videos, archives, and more</small>
          </button>
          {paths.length > 0 && <div className="selected-file-list">
            <div className="selected-file-label">SELECTED FILES <span>{paths.length}</span></div>
            {paths.slice(0, 6).map((p) => <div className="selected-file" key={p}><File size={15} /><span title={p}>{basename(p)}</span><button onClick={() => removeFile(p)} disabled={busy} title="Remove file" aria-label={`Remove ${basename(p)}`}><X size={14}/></button></div>)}
            {paths.length > 6 && <small>+ {paths.length - 6} more files</small>}
          </div>}
          <div className="step-header second"><span className="step-number">02</span><h2>Choose an outcome</h2><span className="step-sub">A little magic, no guesswork</span></div>
          <div className="recipe-grid">{RECIPES.map((r) => {
            const Icon = recipeArt[r.id]; return <button type="button" key={r.id} className={`recipe-card ${r.accent} ${recipe === r.id ? "selected" : ""}`} onClick={() => {setRecipe(r.id);invalidate();}} disabled={busy} aria-pressed={recipe === r.id}>
              <div className="recipe-top"><span className="recipe-icon"><Icon size={20}/></span><span className="radio-indicator">{recipe === r.id && <CheckCircle2 size={18}/>}</span></div><strong>{r.name}</strong><p>{r.description}</p><div className="recipe-detail">{r.detail}</div>
            </button>;
          })}</div>
          <div className="step-header third"><span className="step-number">03</span><h2>Preview & run</h2><span className="step-sub">Never overwrite original files</span></div>
          <div className="run-panel"><div className="run-top"><div><span className="minor-label">YOUR RECIPE</span><strong>{recipeInfo.name}</strong></div><span className="safe-chip"><LockKeyhole size={13}/> Copy-only</span></div>
            <div className="output-row"><div className="output-icon"><FolderOpen size={19}/></div><div className="output-info"><span>Save results to</span><strong title={destination}>{destination ? destination : "Choose a destination folder"}</strong></div><button className="folder-button" onClick={chooseFolder} disabled={busy}>Browse <ChevronDown size={14}/></button></div>
            {error && <div className="error-message" role="alert"><CircleHelp size={17}/>{error}</div>}
            {stage === "preview" && preview && <div className="preview-area"><div className="preview-head"><strong><CheckCircle2 size={16}/> Ready to copy</strong><span>{preview.items.length} files · {formatBytes(preview.total_bytes)}</span></div><div className="preview-items">{preview.items.slice(0, 5).map((item, i) => <div className="preview-item" key={`${item.source}-${i}`}><span title={item.source}>{basename(item.source)}</span><ArrowRight size={13}/><span title={displayPath(item.destination)}>{displayPath(item.destination)}</span></div>)}</div><details className="plan-details"><summary>Inspect every planned path</summary><div className="preview-items">{preview.items.slice(0, visibleCount).map((item, i) => <div className="preview-item" key={`${item.source}-${i}`}><span title={displayPath(item.source)}>{displayPath(item.source)}</span><ArrowRight size={13}/><span title={displayPath(item.destination)}>{displayPath(item.destination)}</span></div>)}</div><p className="plan-count">Showing {Math.min(visibleCount, preview.items.length)} of {preview.items.length}</p>{visibleCount < preview.items.length && <button type="button" className="load-plan" onClick={() => setVisibleCount((count) => count + 100)}>Show next 100</button>}</details>{preview.warnings.map((warn,i) => <div className="warning" key={i}>{warn}</div>)}</div>}
            {stage === "done" && result && <div className={`success-message${result.failed.length ? " has-failures" : ""}`} role={result.failed.length ? "alert" : "status"}>{result.failed.length ? <CircleHelp size={19}/> : <CheckCircle2 size={19}/>}<div><strong>{result.failed.length && result.completed.length === 0 ? "No files copied" : `${result.completed.length} file(s) copied`}</strong><span>{result.failed.length ? `${result.failed.length} failure${result.failed.length === 1 ? "" : "s"}. Review the list below.` : "Completed. Your original files were not modified."}</span></div></div>}
            {stage === "done" && result && result.failed.length > 0 && <ul className="failure-list" aria-label="Files that could not be copied">{result.failed.map((failure, i) => <li key={i}>{failure}</li>)}</ul>}
            <div className="run-footer"><div className="safety-note"><ShieldCheck size={17}/><div><strong>Safe by default</strong><span>Outputs are created as copies.</span></div></div><div className="action-buttons">{stage === "preview" ? <button className="main-button" disabled={busy} onClick={runWork}><Play size={15}/>{busy ? "Processing…" : "Run recipe"}</button> : <button className="main-button" disabled={busy || paths.length === 0 || !destination} onClick={previewWork}><span>{busy ? "Preparing…" : "Preview workflow"}</span><ArrowRight size={17}/></button>}</div></div>
          </div>
          <div className="bottom-note"><MousePointer2 size={15}/> <span>First release: local file workflows. Finder, Explorer, and contextual radial activation arrive in later phases.</span></div>
        </div>
      </main>
    </div>
  );
}
