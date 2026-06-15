import type { AgentRunWithEvents, AgentRuntimeStatus, Brand, Project } from "../types/joi";

type AgentPanelProps = {
  selectedBrand: Brand | null;
  selectedProject: Project | null;
  activityLog: string[];
  agentGoalDraft: string;
  agentRuntimeStatus: AgentRuntimeStatus | null;
  agentRuns: AgentRunWithEvents[];
  latestAgentRun: AgentRunWithEvents | null;
  onAgentGoalChange: (value: string) => void;
  onStartAgentPlan: () => void;
  startingAgentPlan: boolean;
};

export function AgentPanel({
  selectedBrand,
  selectedProject,
  activityLog,
  agentGoalDraft,
  agentRuntimeStatus,
  agentRuns,
  latestAgentRun,
  onAgentGoalChange,
  onStartAgentPlan,
  startingAgentPlan,
}: AgentPanelProps) {
  const runtimeName = agentRuntimeStatus?.runtime_kind === "hermes_core" ? "Hermes Core" : "Unknown";
  const statusLabel = agentRuntimeStatus?.ready ? "Ready" : "Not ready";

  return (
    <aside aria-label="Agent workspace" className="agent-panel">
      <section className="panel-section">
        <p className="eyebrow">Agent</p>
        <h2>Runtime</h2>
        <dl className="compact-list">
          <div>
            <dt>Core</dt>
            <dd>{runtimeName}</dd>
          </div>
          <div>
            <dt>Mode</dt>
            <dd>{agentRuntimeStatus?.runtime_mode ?? "--"}</dd>
          </div>
          <div>
            <dt>Version</dt>
            <dd>{agentRuntimeStatus?.hermes_version || "--"}</dd>
          </div>
          <div>
            <dt>Status</dt>
            <dd>{statusLabel}</dd>
          </div>
        </dl>
        <p className="muted">{agentRuntimeStatus?.message ?? "Runtime status is loading."}</p>
      </section>

      <section className="panel-section">
        <h3>Task Run</h3>
        <form
          className="agent-form"
          onSubmit={(event) => {
            event.preventDefault();
            onStartAgentPlan();
          }}
        >
          <label>
            Agent goal
            <textarea
              onChange={(event) => onAgentGoalChange(event.target.value)}
              rows={4}
              value={agentGoalDraft}
            />
          </label>
          <button disabled={!selectedProject || startingAgentPlan || !agentGoalDraft.trim()} type="submit">
            {startingAgentPlan ? "Starting..." : "Start Plan"}
          </button>
        </form>
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
        <h3>Latest Run</h3>
        {latestAgentRun ? (
          <div className="run-summary">
            <p>{latestAgentRun.run.result_summary}</p>
            <ol className="event-log">
              {latestAgentRun.events.map((event) => (
                <li key={event.id}>
                  <span>{event.role}</span>
                  {event.message}
                </li>
              ))}
            </ol>
          </div>
        ) : (
          <p className="muted">No agent runs for this project.</p>
        )}
      </section>

      <section className="panel-section">
        <h3>Run History</h3>
        <p className="muted">{agentRuns.length} saved runs</p>
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
