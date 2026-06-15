import { useEffect, useMemo, useRef, useState, type FormEvent } from "react";

import type {
  DeliveryPackagePreview,
  DeliveryReport,
  DeliveryReportUpdateInput,
  Project,
} from "../types/joi";

export type DeliveryDraft = {
  user_direction: string;
  export_dir: string;
};

type DeliveryEditDraft = {
  title: string;
  markdown: string;
  is_final_candidate: boolean;
};

const emptyEditDraft: DeliveryEditDraft = {
  title: "",
  markdown: "",
  is_final_candidate: false,
};

type DeliveryWorkspaceProps = {
  deliveryDraft: DeliveryDraft;
  deliveryReports: DeliveryReport[];
  exportingDeliveryPackage: boolean;
  generatingDeliveryReport: boolean;
  onDeliveryDraftChange: (field: keyof DeliveryDraft, value: string) => void;
  onExportDeliveryPackage: (reportId: string) => void;
  onGenerateDeliveryReport: () => void;
  onPreviewDeliveryPackage: (reportId: string) => void;
  onUpdateDeliveryReport: (input: DeliveryReportUpdateInput) => void;
  packagePreview: DeliveryPackagePreview | null;
  previewingDeliveryPackage: boolean;
  savingDeliveryReportId: string | null;
  selectedProject: Project | null;
};

export function DeliveryWorkspace({
  deliveryDraft,
  deliveryReports,
  exportingDeliveryPackage,
  generatingDeliveryReport,
  onDeliveryDraftChange,
  onExportDeliveryPackage,
  onGenerateDeliveryReport,
  onPreviewDeliveryPackage,
  onUpdateDeliveryReport,
  packagePreview,
  previewingDeliveryPackage,
  savingDeliveryReportId,
  selectedProject,
}: DeliveryWorkspaceProps) {
  const [selectedReportId, setSelectedReportId] = useState<string | null>(null);
  const editDraftRef = useRef<DeliveryEditDraft>(emptyEditDraft);
  const finalCandidateInputRef = useRef<HTMLInputElement | null>(null);
  const markdownTextareaRef = useRef<HTMLTextAreaElement | null>(null);
  const titleInputRef = useRef<HTMLInputElement | null>(null);
  const [editDraft, setEditDraft] = useState<DeliveryEditDraft>(emptyEditDraft);
  const selectedReport = useMemo(
    () =>
      deliveryReports.find((report) => report.id === selectedReportId) ??
      deliveryReports[0] ??
      null,
    [deliveryReports, selectedReportId],
  );

  useEffect(() => {
    if (!selectedReport && selectedReportId) {
      setSelectedReportId(null);
    }
    if (selectedReport && selectedReport.id !== selectedReportId) {
      setSelectedReportId(selectedReport.id);
    }
  }, [selectedReport, selectedReportId]);

  useEffect(() => {
    if (selectedReport) {
      const nextDraft = {
        title: selectedReport.title,
        markdown: selectedReport.markdown,
        is_final_candidate: selectedReport.is_final_candidate,
      };
      editDraftRef.current = nextDraft;
      setEditDraft(nextDraft);
    }
  }, [selectedReport?.id, selectedReport?.updated_at]);

  function updateEditDraft(field: keyof DeliveryEditDraft, value: string | boolean) {
    setEditDraft((draft) => {
      const nextDraft = { ...draft, [field]: value };
      editDraftRef.current = nextDraft;
      return nextDraft;
    });
  }

  function submitReportUpdate() {
    if (!selectedReport) {
      return;
    }
    const currentDraft = {
      title: titleInputRef.current?.value ?? editDraftRef.current.title,
      markdown: markdownTextareaRef.current?.value ?? editDraftRef.current.markdown,
      is_final_candidate:
        finalCandidateInputRef.current?.checked ?? editDraftRef.current.is_final_candidate,
    };
    onUpdateDeliveryReport({
      id: selectedReport.id,
      title: currentDraft.title,
      markdown: currentDraft.markdown,
      sections_json: selectedReport.sections_json,
      is_final_candidate: currentDraft.is_final_candidate,
    });
  }

  const canGenerate = Boolean(selectedProject) && !generatingDeliveryReport;
  const canPreview = Boolean(selectedProject && selectedReport) && !previewingDeliveryPackage;
  const canExport =
    Boolean(selectedProject && selectedReport && deliveryDraft.export_dir.trim()) &&
    !exportingDeliveryPackage;

  return (
    <div className="delivery-layout">
      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Delivery Package</h2>
          <span className="muted">
            {deliveryReports.length > 0
              ? `${deliveryReports.length} report(s)`
              : "No delivery report yet"}
          </span>
        </div>

        <form className="delivery-toolbar" onSubmit={submit(onGenerateDeliveryReport)}>
          <label className="wide-field">
            Report direction
            <textarea
              disabled={!selectedProject || generatingDeliveryReport}
              onChange={(event) => onDeliveryDraftChange("user_direction", event.target.value)}
              placeholder="Prepare final handoff for creative and production."
              rows={3}
              value={deliveryDraft.user_direction}
            />
          </label>
          <button disabled={!canGenerate} type="submit">
            {generatingDeliveryReport ? "Generating" : "Generate Delivery Report"}
          </button>
        </form>

        <div className="delivery-preview-grid">
          <PreviewMetric label="Project JSON" value={packagePreview?.project_json_file_name ?? "--"} />
          <PreviewMetric label="Assets Folder" value={packagePreview?.assets_folder_name ?? "--"} />
          <PreviewMetric
            label="Markdown Report"
            value={packagePreview?.delivery_report_file_name ?? "--"}
          />
          <PreviewMetric
            label="Included"
            value={
              packagePreview
                ? `${packagePreview.included_storyboards_count} storyboards · ${packagePreview.included_prompt_packages_count} prompts`
                : "--"
            }
          />
        </div>

        {packagePreview?.warnings.length ? (
          <div className="delivery-warning-list">
            {packagePreview.warnings.map((warning) => (
              <span key={warning}>{warning}</span>
            ))}
          </div>
        ) : null}
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Report Editor</h2>
          <div className="prompt-actions">
            <button
              disabled={!canPreview}
              onClick={() => selectedReport && onPreviewDeliveryPackage(selectedReport.id)}
              type="button"
            >
              {previewingDeliveryPackage ? "Previewing" : "Preview Package"}
            </button>
            <button
              disabled={!selectedReport || savingDeliveryReportId === selectedReport.id}
              onClick={submitReportUpdate}
              type="button"
            >
              {selectedReport && savingDeliveryReportId === selectedReport.id ? "Saving" : "Save Report"}
            </button>
          </div>
        </div>

        {selectedReport ? (
          <div className="delivery-editor-grid">
            <div className="delivery-report-list">
              {deliveryReports.map((report) => (
                <button
                  className={report.id === selectedReport.id ? "rail-item selected" : "rail-item"}
                  key={report.id}
                  onClick={() => setSelectedReportId(report.id)}
                  type="button"
                >
                  <span>{report.title}</span>
                  <small>{report.is_final_candidate ? "Final candidate" : "Draft report"}</small>
                </button>
              ))}
            </div>

            <form className="delivery-edit-form" onSubmit={submit(submitReportUpdate)}>
              <label>
                Report title
                <input
                  onChange={(event) => updateEditDraft("title", event.target.value)}
                  ref={titleInputRef}
                  value={editDraft.title}
                />
              </label>
              <label className="checkbox-label">
                <input
                  checked={editDraft.is_final_candidate}
                  onChange={(event) =>
                    updateEditDraft("is_final_candidate", event.target.checked)
                  }
                  ref={finalCandidateInputRef}
                  type="checkbox"
                />
                Final candidate
              </label>
              <label className="wide-field">
                Markdown report
                <textarea
                  onChange={(event) => updateEditDraft("markdown", event.target.value)}
                  ref={markdownTextareaRef}
                  rows={18}
                  value={editDraft.markdown}
                />
              </label>
            </form>
          </div>
        ) : (
          <p className="muted">Generate a delivery report to edit Markdown and package settings.</p>
        )}
      </section>

      <section className="workspace-panel wide">
        <h2>Export</h2>
        <form
          className="delivery-export-form"
          onSubmit={submit(() => selectedReport && onExportDeliveryPackage(selectedReport.id))}
        >
          <label>
            Export directory
            <input
              onChange={(event) => onDeliveryDraftChange("export_dir", event.target.value)}
              placeholder="D:/exports/joi-project"
              value={deliveryDraft.export_dir}
            />
          </label>
          <button disabled={!canExport} type="submit">
            {exportingDeliveryPackage ? "Exporting" : "Export Package"}
          </button>
        </form>
      </section>
    </div>
  );
}

function PreviewMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function submit(handler: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    handler();
  };
}
