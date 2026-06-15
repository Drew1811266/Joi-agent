import type { Brand, HealthResponse, Project } from "../types/joi";

type TopBarProps = {
  health: HealthResponse | null;
  selectedBrand: Brand | null;
  selectedProject: Project | null;
  onSaveSnapshot: () => void;
  savingSnapshot: boolean;
};

export function TopBar({
  health,
  selectedBrand,
  selectedProject,
  onSaveSnapshot,
  savingSnapshot,
}: TopBarProps) {
  return (
    <header className="top-bar">
      <div className="brand-mark">
        <span className="brand-dot" aria-hidden="true" />
        <div>
          <p>Joi Agent</p>
          <strong>Fashion advertising workspace</strong>
        </div>
      </div>
      <div className="top-context">
        <span className={health?.status === "ready" ? "status ready" : "status"}>
          Backend: {health?.status ?? "checking"}
        </span>
        <span>{selectedBrand?.name ?? "No brand selected"}</span>
        <span>{selectedProject?.title ?? "No project selected"}</span>
      </div>
      <div className="top-actions">
        <button disabled={!selectedProject || savingSnapshot} onClick={onSaveSnapshot} type="button">
          {savingSnapshot ? "Saving" : "Save Snapshot"}
        </button>
        <button disabled type="button">
          Export Project
        </button>
      </div>
    </header>
  );
}
