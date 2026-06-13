import React, { useState } from "react";
import Upload from "./pages/Upload";
import Visualizer from "./pages/Visualizer";
import { WasmAnalysisResult } from "./types";

type Page = "upload" | "visualizer";

const App: React.FC = () => {
  const [page, setPage] = useState<Page>("upload");
  const [result, setResult] = useState<WasmAnalysisResult | null>(null);
  const [fileName, setFileName] = useState<string>("");

  const handleAnalysisComplete = (analysis: WasmAnalysisResult, name: string) => {
    setResult(analysis);
    setFileName(name);
    setPage("visualizer");
  };

  const handleBack = () => {
    setPage("upload");
    setResult(null);
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
      <header className="bg-qsae-700 text-white py-4 px-6 shadow-lg">
        <div className="max-w-6xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-white rounded-lg flex items-center justify-center">
              <span className="text-qsae-700 font-bold text-lg">Q</span>
            </div>
            <div>
              <h1 className="text-xl font-bold">QSAE Web Demo</h1>
              <p className="text-xs text-qsae-200">Quorum Sensing Adaptive Encoder</p>
            </div>
          </div>
          <div className="text-sm text-qsae-200">Phase 5 — Interactive WASM Demo</div>
        </div>
      </header>

      <main className="max-w-6xl mx-auto p-6">
        {page === "upload" ? (
          <Upload onAnalysisComplete={handleAnalysisComplete} />
        ) : (
          <Visualizer result={result} fileName={fileName} onBack={handleBack} />
        )}
      </main>
    </div>
  );
};

export default App;
