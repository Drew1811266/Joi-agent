import type { FormEvent } from "react";

import type {
  Project,
  QualityReview,
  QualityReviewCheck,
  QualityReviewSuggestion,
} from "../types/joi";

export type ReviewDraft = {
  user_direction: string;
};

type ReviewWorkspaceProps = {
  applyingSuggestionId: string | null;
  generatingQualityReview: boolean;
  latestChecks: QualityReviewCheck[];
  latestSuggestions: QualityReviewSuggestion[];
  onApplySuggestion: (reviewId: string, suggestionId: string) => void;
  onReviewDraftChange: (field: keyof ReviewDraft, value: string) => void;
  onSubmitReview: () => void;
  qualityReviews: QualityReview[];
  reviewDraft: ReviewDraft;
  selectedProject: Project | null;
};

export function ReviewWorkspace({
  applyingSuggestionId,
  generatingQualityReview,
  latestChecks,
  latestSuggestions,
  onApplySuggestion,
  onReviewDraftChange,
  onSubmitReview,
  qualityReviews,
  reviewDraft,
  selectedProject,
}: ReviewWorkspaceProps) {
  const latestReview = qualityReviews[qualityReviews.length - 1] ?? null;
  const checks = latestChecks.length > 0 ? latestChecks : normalizeChecks(latestReview);
  const suggestions =
    latestSuggestions.length > 0 ? latestSuggestions : normalizeSuggestions(latestReview);

  return (
    <div className="review-layout">
      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Quality Review</h2>
          <span className="muted">{latestReview ? "Latest review" : "No review yet"}</span>
        </div>
        <form className="review-generator-form" onSubmit={submit(onSubmitReview)}>
          <label className="wide-field">
            Review direction
            <textarea
              disabled={!selectedProject || generatingQualityReview}
              onChange={(event) => onReviewDraftChange("user_direction", event.target.value)}
              placeholder="Check prompt completeness before delivery."
              rows={3}
              value={reviewDraft.user_direction}
            />
          </label>
          <button disabled={!selectedProject || generatingQualityReview} type="submit">
            {generatingQualityReview ? "Generating" : "Generate Review"}
          </button>
        </form>
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Review Checklist</h2>
          <span className="muted">{checks.length} check(s)</span>
        </div>
        {latestReview ? <strong className="review-score">{latestReview.score}/100</strong> : null}
        {latestReview ? <p className="muted">{latestReview.summary}</p> : null}
        {checks.length > 0 ? (
          <div className="review-check-list">
            {checks.map((check) => (
              <article className={`review-check ${check.status}`} key={check.id}>
                <div>
                  <strong>{check.title}</strong>
                  <span>{check.message}</span>
                </div>
                <small>
                  {check.category} · {check.severity}
                </small>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted">No checklist records.</p>
        )}
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Revision Suggestions</h2>
          <span className="muted">{suggestions.length} suggestion(s)</span>
        </div>
        {suggestions.length > 0 && latestReview ? (
          <div className="review-suggestion-list">
            {suggestions.map((suggestion) => (
              <article className="review-suggestion" key={suggestion.id}>
                <div className="review-suggestion-copy">
                  <strong>{targetLabel(suggestion)}</strong>
                  <p>{suggestion.suggested_value}</p>
                  <small>{suggestion.rationale}</small>
                </div>
                <button
                  disabled={suggestion.status !== "pending" || applyingSuggestionId === suggestion.id}
                  onClick={() => onApplySuggestion(latestReview.id, suggestion.id)}
                  type="button"
                >
                  {suggestion.status === "applied"
                    ? "Applied"
                    : applyingSuggestionId === suggestion.id
                      ? "Applying"
                      : "Apply Suggestion"}
                </button>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted">No revision suggestions.</p>
        )}
      </section>
    </div>
  );
}

function normalizeChecks(review: QualityReview | null): QualityReviewCheck[] {
  return Array.isArray(review?.checklist_json)
    ? (review.checklist_json as QualityReviewCheck[])
    : [];
}

function normalizeSuggestions(review: QualityReview | null): QualityReviewSuggestion[] {
  return Array.isArray(review?.suggestions_json)
    ? (review.suggestions_json as QualityReviewSuggestion[])
    : [];
}

function targetLabel(suggestion: QualityReviewSuggestion): string {
  return `${suggestion.target_type} · ${suggestion.field}`;
}

function submit(handler: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    handler();
  };
}
