import type { FormEvent } from "react";

import type { BetaWorkflowStatusResult, Project } from "../types/joi";

export type BetaWorkflowDraft = {
  user_direction: string;
  image_brief: string;
  reference_title: string;
  reference_url: string;
  reference_excerpt: string;
  memory_feedback: string;
  save_snapshot: boolean;
};

type BetaWorkflowPanelProps = {
  betaDraft: BetaWorkflowDraft;
  betaStatus: BetaWorkflowStatusResult | null;
  onBetaDraftChange: (field: keyof BetaWorkflowDraft, value: string | boolean) => void;
  onRunBetaWorkflow: () => void;
  runningBetaWorkflow: boolean;
  selectedProject: Project | null;
};

export function BetaWorkflowPanel({
  betaDraft,
  betaStatus,
  onBetaDraftChange,
  onRunBetaWorkflow,
  runningBetaWorkflow,
  selectedProject,
}: BetaWorkflowPanelProps) {
  return (
    <section className="workspace-panel wide">
      <div className="section-heading">
        <h2>Beta Workflow</h2>
        <span className={betaStatus?.ready ? "status-pill complete" : "status-pill"}>
          {betaStatus?.ready ? "Beta ready" : `${betaStatus?.score ?? 0}/110`}
        </span>
      </div>
      <form className="beta-workflow-form" onSubmit={submit(onRunBetaWorkflow)}>
        <label>
          Beta direction
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("user_direction", event.target.value)}
            rows={3}
            value={betaDraft.user_direction}
          />
        </label>
        <label>
          Beta image brief
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("image_brief", event.target.value)}
            rows={3}
            value={betaDraft.image_brief}
          />
        </label>
        <label>
          Reference title
          <input
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_title", event.target.value)}
            value={betaDraft.reference_title}
          />
        </label>
        <label>
          Reference URL
          <input
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_url", event.target.value)}
            value={betaDraft.reference_url}
          />
        </label>
        <label className="wide-field">
          Reference excerpt
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_excerpt", event.target.value)}
            rows={3}
            value={betaDraft.reference_excerpt}
          />
        </label>
        <label className="wide-field">
          Memory feedback
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("memory_feedback", event.target.value)}
            rows={2}
            value={betaDraft.memory_feedback}
          />
        </label>
        <label className="checkbox-row">
          <input
            checked={betaDraft.save_snapshot}
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("save_snapshot", event.target.checked)}
            type="checkbox"
          />
          Save beta snapshot
        </label>
        <button disabled={!selectedProject || runningBetaWorkflow} type="submit">
          {runningBetaWorkflow ? "Running" : "Run Beta Workflow"}
        </button>
      </form>
      {betaStatus ? (
        <div className="beta-step-list">
          {betaStatus.steps.map((step) => (
            <article className={`beta-step ${step.status}`} key={step.id}>
              <div>
                <strong>{step.title}</strong>
                <span>{step.message}</span>
              </div>
              <small>
                {step.source_count} source(s) · {step.target_tab}
              </small>
            </article>
          ))}
        </div>
      ) : (
        <p className="muted">Select a project to inspect beta readiness.</p>
      )}
    </section>
  );
}

function submit(handler: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    handler();
  };
}
