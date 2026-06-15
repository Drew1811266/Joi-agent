import type { FormEvent } from "react";

import type {
  Project,
  ResearchReport,
  ResearchReportResult,
  ResearchSourceInput,
} from "../types/joi";

export type ResearchDraft = {
  research_goal: string;
  market_focus: string;
  platform_focus_text: string;
  source_title: string;
  source_url: string;
  source_type: string;
  source_excerpt: string;
};

type ResearchWorkspaceProps = {
  generatingResearch: boolean;
  onResearchDraftChange: (field: keyof ResearchDraft, value: string) => void;
  onSubmitResearchReport: () => void;
  researchDraft: ResearchDraft;
  researchReports: ResearchReport[];
  researchResult: ResearchReportResult | null;
  selectedProject: Project | null;
};

export function ResearchWorkspace({
  generatingResearch,
  onResearchDraftChange,
  onSubmitResearchReport,
  researchDraft,
  researchReports,
  researchResult,
  selectedProject,
}: ResearchWorkspaceProps) {
  return (
    <div className="research-layout">
      <section className="workspace-panel wide">
        <h2>Research Brief</h2>
        <form className="brief-form" onSubmit={submit(onSubmitResearchReport)}>
          <label className="wide-field">
            Research goal
            <textarea
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("research_goal", event.target.value)}
              placeholder="Find fashion ad references for a 15s trench coat launch film"
              rows={3}
              value={researchDraft.research_goal}
            />
          </label>
          <label>
            Market focus
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("market_focus", event.target.value)}
              placeholder="urban commuter outerwear"
              value={researchDraft.market_focus}
            />
          </label>
          <label>
            Platform focus
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("platform_focus_text", event.target.value)}
              placeholder="jimeng_video, grok_video"
              value={researchDraft.platform_focus_text}
            />
          </label>
          <label>
            Source title
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_title", event.target.value)}
              placeholder="Reference campaign note"
              value={researchDraft.source_title}
            />
          </label>
          <label>
            Source URL
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_url", event.target.value)}
              placeholder="https://example.com/reference"
              value={researchDraft.source_url}
            />
          </label>
          <label>
            Source type
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_type", event.target.value)}
              placeholder="reference"
              value={researchDraft.source_type}
            />
          </label>
          <label className="wide-field">
            Source excerpt
            <textarea
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_excerpt", event.target.value)}
              placeholder="Close fabric texture and walking movement made the product benefit clear."
              rows={4}
              value={researchDraft.source_excerpt}
            />
          </label>
          <button
            disabled={!selectedProject || generatingResearch || !canSubmitResearch(researchDraft)}
            type="submit"
          >
            {generatingResearch ? "Generating" : "Generate Research Report"}
          </button>
        </form>
      </section>

      <section className="workspace-panel">
        <h2>Latest Research</h2>
        {researchResult ? (
          <div className="understanding-result">
            <p>{researchResult.report.summary}</p>
            <dl className="compact-list result-list">
              <div>
                <dt>Sources</dt>
                <dd>{researchResult.sources.length}</dd>
              </div>
              <div>
                <dt>Findings</dt>
                <dd>{researchResult.findings.length}</dd>
              </div>
            </dl>
            <div className="data-list compact-data-list">
              {researchResult.findings.map((finding) => (
                <article className="data-row" key={`${finding.source_index}-${finding.title}`}>
                  <strong>{finding.title}</strong>
                  <span>{finding.insight}</span>
                  <small>{finding.creative_implication}</small>
                </article>
              ))}
            </div>
          </div>
        ) : (
          <p className="muted">Generated research findings will appear here.</p>
        )}
      </section>

      <section className="workspace-panel">
        <h2>Saved Reports</h2>
        {researchReports.length === 0 ? (
          <p className="muted">No research reports yet.</p>
        ) : (
          <div className="data-list compact-data-list">
            {researchReports.map((report) => (
              <article className="data-row" key={report.id}>
                <strong>{report.summary}</strong>
                <span>{sourceCount(report.sources_json)} source(s)</span>
                <small>{report.created_at}</small>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}

export function researchSourceFromDraft(draft: ResearchDraft): ResearchSourceInput {
  return {
    title: draft.source_title,
    url: draft.source_url,
    source_type: draft.source_type || "reference",
    excerpt: draft.source_excerpt,
  };
}

function canSubmitResearch(draft: ResearchDraft): boolean {
  return Boolean(draft.research_goal.trim() && draft.source_title.trim() && draft.source_excerpt.trim());
}

function sourceCount(value: unknown): number {
  return Array.isArray(value) ? value.length : 0;
}

function submit(action: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    action();
  };
}
