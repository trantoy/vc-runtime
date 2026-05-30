function renderPipeline(targetId, stages) {
  const target = document.getElementById(targetId);
  if (!target) return;

  const width = 1040;
  const height = 180;
  const gap = 18;
  const stageWidth = Math.floor((width - gap * (stages.length + 1)) / stages.length);
  const stageHeight = 74;
  const y = 52;

  const nodes = stages
    .map((stage, index) => {
      const x = gap + index * (stageWidth + gap);
      const color = stage.kind === "risk" ? "#fff2df" : stage.kind === "metric" ? "#e7f5ef" : "#e8f2fb";
      const stroke = stage.kind === "risk" ? "#ad6418" : stage.kind === "metric" ? "#21866f" : "#1267b3";
      const arrow =
        index < stages.length - 1
          ? `<path d="M ${x + stageWidth + 4} ${y + stageHeight / 2} L ${x + stageWidth + gap - 6} ${y + stageHeight / 2}" stroke="#8290a3" stroke-width="2" marker-end="url(#arrow)" />`
          : "";
      return `
        <rect x="${x}" y="${y}" width="${stageWidth}" height="${stageHeight}" rx="8" fill="${color}" stroke="${stroke}" />
        <text x="${x + 12}" y="${y + 28}" fill="#17202a" font-size="15" font-weight="650">${stage.title}</text>
        <text x="${x + 12}" y="${y + 52}" fill="#5d6978" font-size="12">${stage.note}</text>
        ${arrow}
      `;
    })
    .join("");

  target.innerHTML = `
    <svg viewBox="0 0 ${width} ${height}" role="img" aria-label="vc-runtime pipeline" width="100%" height="100%">
      <defs>
        <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" fill="#8290a3" />
        </marker>
      </defs>
      ${nodes}
    </svg>
  `;
}

window.addEventListener("DOMContentLoaded", () => {
  renderPipeline("architecture-pipeline", [
    { title: "Input", note: "callback + ring", kind: "stage" },
    { title: "DSP", note: "resample + chunks", kind: "stage" },
    { title: "Model", note: "RVC ONNX", kind: "risk" },
    { title: "Output", note: "SOLA + buffer", kind: "stage" },
    { title: "Metrics", note: "p95 + queues", kind: "metric" },
  ]);

  renderPipeline("phase0-pipeline", [
    { title: "Passthrough", note: "CPAL proof", kind: "metric" },
    { title: "Offline RVC", note: "ONNX proof", kind: "risk" },
    { title: "Metrics", note: "schema", kind: "stage" },
    { title: "Provider", note: "probe", kind: "risk" },
    { title: "Baseline", note: "compare", kind: "stage" },
  ]);
});
