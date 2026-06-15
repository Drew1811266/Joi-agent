import { EmptyState } from "./EmptyState";
import type { BriefUnderstandingResult, CreativeDirection, ProductUnderstanding } from "../types/joi";

type UnderstandingResultPanelProps = {
  creativeDirections: CreativeDirection[];
  productUnderstandings: ProductUnderstanding[];
  result: BriefUnderstandingResult | null;
};

export function UnderstandingResultPanel({
  creativeDirections,
  productUnderstandings,
  result,
}: UnderstandingResultPanelProps) {
  return (
    <section className="workspace-panel wide">
      <h2>Structured Context</h2>
      {result ? (
        <div className="understanding-result">
          <dl className="compact-list result-list">
            <div>
              <dt>Brief</dt>
              <dd>{result.brief_summary}</dd>
            </div>
            <div>
              <dt>Brand</dt>
              <dd>{result.brand_summary}</dd>
            </div>
            <div>
              <dt>Visual</dt>
              <dd>{result.visual_direction}</dd>
            </div>
          </dl>
          <TagList label="Selling Points" values={result.selling_points} />
          <TagList label="Constraints" values={result.constraints} />
          {result.missing_questions.length > 0 ? (
            <div className="question-list">
              <h3>Missing Questions</h3>
              <ul>
                {result.missing_questions.map((question) => (
                  <li key={question}>{question}</li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>
      ) : null}

      {!result && productUnderstandings.length === 0 && creativeDirections.length === 0 ? (
        <EmptyState body="Generated product understanding and creative direction will appear here." title="No context yet" />
      ) : null}

      {productUnderstandings.length > 0 ? (
        <div className="data-list">
          <h3>Product Understandings</h3>
          {productUnderstandings.map((understanding) => (
            <article className="data-row" key={understanding.id}>
              <strong>{understanding.product_name || "Untitled product"}</strong>
              <span>{[understanding.category, understanding.audience].filter(Boolean).join(" · ")}</span>
              <small>{formatJsonList(understanding.selling_points_json)}</small>
            </article>
          ))}
        </div>
      ) : null}

      {creativeDirections.length > 0 ? (
        <div className="data-list">
          <h3>Creative Directions</h3>
          {creativeDirections.map((direction) => (
            <article className="data-row" key={direction.id}>
              <strong>{direction.title}</strong>
              <span>{direction.visual_style}</span>
              <small>{direction.rationale}</small>
            </article>
          ))}
        </div>
      ) : null}
    </section>
  );
}

function TagList({ label, values }: { label: string; values: string[] }) {
  if (values.length === 0) {
    return null;
  }
  return (
    <div className="tag-group">
      <h3>{label}</h3>
      <div className="tag-list">
        {values.map((value) => (
          <span key={value}>{value}</span>
        ))}
      </div>
    </div>
  );
}

function formatJsonList(value: unknown): string {
  if (Array.isArray(value)) {
    return value.filter((item) => typeof item === "string").join(", ");
  }
  return "";
}
