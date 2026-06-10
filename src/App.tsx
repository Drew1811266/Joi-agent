import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

type HealthResponse = {
  status: string;
  app_name: string;
  phase: string;
};

export default function App() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<HealthResponse>("joi_health_check")
      .then(setHealth)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <main className="app-shell">
      <section className="status-panel">
        <p className="eyebrow">Joi Agent</p>
        <h1>Phase 1 local data store</h1>
        <dl>
          <div>
            <dt>Backend</dt>
            <dd>{health ? health.status : "checking"}</dd>
          </div>
          <div>
            <dt>App</dt>
            <dd>{health ? health.app_name : "Joi Agent"}</dd>
          </div>
          <div>
            <dt>Phase</dt>
            <dd>{health ? health.phase : "Phase 1"}</dd>
          </div>
        </dl>
        {error ? <p className="error">{error}</p> : null}
      </section>
    </main>
  );
}
