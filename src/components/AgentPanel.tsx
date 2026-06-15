import type { Brand, Project } from "../types/joi";

type AgentPanelProps = {
  selectedBrand: Brand | null;
  selectedProject: Project | null;
  activityLog: string[];
};

export function AgentPanel({ selectedBrand, selectedProject, activityLog }: AgentPanelProps) {
  return (
    <aside aria-label="Agent workspace" className="agent-panel">
      <section className="panel-section">
        <p className="eyebrow">Agent</p>
        <h2>Execution standby</h2>
        <p className="muted">
          Joi can read this workspace context. Planning and generation runs will be wired in the
          agent runtime milestone.
        </p>
      </section>

      <section className="panel-section">
        <h3>Current Task</h3>
        <p>{selectedProject ? `Prepare workspace for ${selectedProject.title}` : "Select a project"}</p>
      </section>

      <section className="panel-section">
        <h3>Context</h3>
        <dl className="compact-list">
          <div>
            <dt>Brand</dt>
            <dd>{selectedBrand?.name ?? "None"}</dd>
          </div>
          <div>
            <dt>Project</dt>
            <dd>{selectedProject?.title ?? "None"}</dd>
          </div>
        </dl>
      </section>

      <section className="panel-section">
        <h3>Activity Log</h3>
        <ol className="activity-log">
          {activityLog.length === 0 ? (
            <li>No activity yet.</li>
          ) : (
            activityLog.slice(-6).map((entry, index) => <li key={`${entry}-${index}`}>{entry}</li>)
          )}
        </ol>
      </section>
    </aside>
  );
}
