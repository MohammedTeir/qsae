import React, { useState, useCallback } from "react";
import { Upload as UploadIcon, FileText, Binary, Zap } from "lucide-react";
import { useWasm } from "../hooks/useWasm";
import { WasmAnalysisResult } from "../types";
import ParamSliders from "../components/ParamSliders";

interface UploadProps {
  onAnalysisComplete: (result: WasmAnalysisResult, fileName: string) => void;
}

const Upload: React.FC<UploadProps> = ({ onAnalysisComplete }) => {
  const { module, loading, error } = useWasm();
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [params, setParams] = useState({ lambda: 0.5, delta: 1.2, block_size: 65536 });
  const [dragActive, setDragActive] = useState(false);

  const handleFile = useCallback(async (file: File) => {
    if (!module || isAnalyzing) return;
    if (file.size > 50 * 1024 * 1024) {
      alert("File too large. Max 50MB for browser demo.");
      return;
    }

    setIsAnalyzing(true);

    try {
      const arrayBuffer = await file.arrayBuffer();
      const data = new Uint8Array(arrayBuffer);

      const result = await module.compressAndAnalyze(data, params.lambda, params.delta, params.block_size);
      onAnalysisComplete(result, file.name);
    } catch (err) {
      console.error("Analysis failed:", err);
      alert("Analysis failed: " + (err instanceof Error ? err.message : String(err)));
    } finally {
      setIsAnalyzing(false);
    }
  }, [module, isAnalyzing, params, onAnalysisComplete]);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);

    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFile(e.dataTransfer.files[0]);
    }
  }, [handleFile]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);
  }, []);

  const handleFileInput = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      handleFile(e.target.files[0]);
    }
  }, [handleFile]);

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20">
        <div className="w-16 h-16 border-4 border-qsae-200 border-t-qsae-600 rounded-full animate-spin" />
        <p className="mt-4 text-gray-500">Loading QSAE WASM module...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center py-20">
        <p className="text-red-600">Failed to load WASM: {error}</p>
      </div>
    );
  }

  return (
    <div className="space-y-8">
      {/* Hero */}
      <div className="text-center py-8">
        <h2 className="text-3xl font-bold mb-3">Interactive Compression Demo</h2>
        <p className="text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          Upload a file and watch QSAE's quorum sensing algorithm work in real-time.
          See entropy heatmaps, quorum signals, and codec switching as they happen.
        </p>
      </div>

      {/* Drop Zone */}
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        className={`border-2 border-dashed rounded-xl p-12 text-center transition-all cursor-pointer ${
          dragActive
            ? "border-qsae-500 bg-qsae-50 dark:bg-qsae-900/20"
            : "border-gray-300 dark:border-gray-600 hover:border-qsae-400"
        }`}
      >
        <input
          type="file"
          onChange={handleFileInput}
          className="hidden"
          id="file-input"
        />
        <label htmlFor="file-input" className="cursor-pointer block">
          <div className="flex flex-col items-center gap-4">
            <div className="w-20 h-20 bg-qsae-100 dark:bg-qsae-900/30 rounded-full flex items-center justify-center">
              <UploadIcon className="w-10 h-10 text-qsae-600" />
            </div>
            <div>
              <p className="text-xl font-medium text-gray-700 dark:text-gray-300">
                Drop a file here
              </p>
              <p className="text-sm text-gray-500 mt-1">
                or click to browse (max 50MB)
              </p>
            </div>
          </div>
        </label>
      </div>

      {/* Sample Files */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[
          { name: "Code Sample", icon: FileText, desc: "Source code with repetition" },
          { name: "JSON Data", icon: Binary, desc: "Structured data patterns" },
          { name: "Mixed Binary", icon: Zap, desc: "Multiple entropy regimes" },
        ].map((sample) => {
          const Icon = sample.icon;
          return (
            <button
              key={sample.name}
              onClick={() => {
                // Generate sample data
                let data: Uint8Array;
                if (sample.name === "Code Sample") {
                  const code = "function hello() { return 'world'; }\n".repeat(1000);
                  data = new TextEncoder().encode(code);
                } else if (sample.name === "JSON Data") {
                  const json = JSON.stringify({ items: Array.from({ length: 500 }, (_, i) => ({ id: i, value: i * 2 })) });
                  data = new TextEncoder().encode(json);
                } else {
                  const mixed = new Uint8Array(30000);
                  mixed.fill(0, 0, 10000);
                  for (let i = 10000; i < 30000; i++) {
                    mixed[i] = (i * 17 + 31) % 256;
                  }
                  data = mixed;
                }

                const file = new File([data], `${sample.name.toLowerCase().replace(' ', '_')}.bin`, { type: 'application/octet-stream' });
                handleFile(file);
              }}
              className="qsae-card p-6 text-left hover:shadow-lg transition-shadow"
            >
              <Icon className="w-8 h-8 text-qsae-600 mb-3" />
              <h3 className="font-semibold">{sample.name}</h3>
              <p className="text-sm text-gray-500 mt-1">{sample.desc}</p>
            </button>
          );
        })}
      </div>

      {/* Parameters */}
      <div className="qsae-card p-6">
        <h3 className="text-lg font-semibold mb-4">Quorum Parameters</h3>
        <ParamSliders params={params} onChange={setParams} />
      </div>

      {/* Analyzing State */}
      {isAnalyzing && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-xl p-8 shadow-2xl">
            <div className="w-12 h-12 border-4 border-qsae-200 border-t-qsae-600 rounded-full animate-spin mx-auto" />
            <p className="mt-4 text-center font-medium">Analyzing with QSAE...</p>
            <p className="text-sm text-gray-500 text-center mt-1">Computing entropy & quorum signals</p>
          </div>
        </div>
      )}
    </div>
  );
};

export default Upload;
