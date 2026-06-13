import React, { useState, useEffect } from "react";
import { ArrowLeft, Download, Info } from "lucide-react";
import { WasmAnalysisResult } from "../types";
import EntropyHeatmap from "../components/EntropyHeatmap";
import QuorumSignal from "../components/QuorumSignal";
import CodecMap from "../components/CodecMap";
import StatsPanel from "../components/StatsPanel";

interface VisualizerProps {
  result: WasmAnalysisResult | null;
  fileName: string;
  onBack: () => void;
}

const Visualizer: React.FC<VisualizerProps> = ({ result, fileName, onBack }) => {
  const [activeTab, setActiveTab] = useState<"entropy" | "quorum" | "codecs" | "stats">("entropy");

  if (!result) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-500">No analysis data available</p>
        <button onClick={onBack} className="mt-4 qsae-btn-primary">
          <ArrowLeft className="w-4 h-4 inline mr-2" />
          Back to Upload
        </button>
      </div>
    );
  }

  const tabs = [
    { id: "entropy" as const, label: "Entropy Heatmap", icon: "🌡️" },
    { id: "quorum" as const, label: "Quorum Signal", icon: "📈" },
    { id: "codecs" as const, label: "Codec Map", icon: "🎨" },
    { id: "stats" as const, label: "Statistics", icon: "📊" },
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button onClick={onBack} className="qsae-btn-secondary flex items-center gap-2">
            <ArrowLeft className="w-4 h-4" />
            Back
          </button>
          <div>
            <h2 className="text-xl font-semibold">Analysis Results</h2>
            <p className="text-sm text-gray-500">{fileName}</p>
          </div>
        </div>
        <div className="flex items-center gap-2 text-sm text-gray-500">
          <Info className="w-4 h-4" />
          <span>{result.block_count} blocks analyzed</span>
        </div>
      </div>

      {/* Summary Cards */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="qsae-card p-4">
          <p className="text-xs text-gray-500 uppercase">Original</p>
          <p className="text-xl font-bold">{formatBytes(result.original_size)}</p>
        </div>
        <div className="qsae-card p-4">
          <p className="text-xs text-gray-500 uppercase">Compressed</p>
          <p className="text-xl font-bold text-qsae-600">{formatBytes(result.compressed_size)}</p>
        </div>
        <div className="qsae-card p-4">
          <p className="text-xs text-gray-500 uppercase">Ratio</p>
          <p className="text-xl font-bold text-green-600">{result.ratio.toFixed(2)}:1</p>
        </div>
        <div className="qsae-card p-4">
          <p className="text-xs text-gray-500 uppercase">Time</p>
          <p className="text-xl font-bold">{(result.duration_ms / 1000).toFixed(2)}s</p>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b border-gray-200 dark:border-gray-700">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 ${
              activeTab === tab.id
                ? "border-qsae-600 text-qsae-600"
                : "border-transparent text-gray-500 hover:text-gray-700"
            }`}
          >
            <span className="mr-2">{tab.icon}</span>
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="qsae-card p-6">
        {activeTab === "entropy" && (
          <div>
            <h3 className="text-lg font-semibold mb-4">Entropy Landscape</h3>
            <p className="text-sm text-gray-500 mb-4">
              Each block's Shannon entropy (0-8 bits/byte). Green = low entropy (compressible), Red = high entropy (random/encrypted).
            </p>
            <EntropyHeatmap data={result.entropy_profile} />
          </div>
        )}

        {activeTab === "quorum" && (
          <div>
            <h3 className="text-lg font-semibold mb-4">Quorum Signal Q(i)</h3>
            <p className="text-sm text-gray-500 mb-4">
              Accumulated entropy signal with exponential decay. Threshold crossings trigger codec switches.
            </p>
            <QuorumSignal data={result.quorum_curve} switchPoints={result.switch_points} />
          </div>
        )}

        {activeTab === "codecs" && (
          <div>
            <h3 className="text-lg font-semibold mb-4">Codec Assignment Map</h3>
            <p className="text-sm text-gray-500 mb-4">
              Color-coded blocks showing which codec was selected for each region.
            </p>
            <CodecMap assignments={result.codec_assignments} names={result.codec_names} counts={result.codec_counts} />
          </div>
        )}

        {activeTab === "stats" && (
          <div>
            <h3 className="text-lg font-semibold mb-4">Detailed Statistics</h3>
            <StatsPanel result={result} />
          </div>
        )}
      </div>

      {/* Download */}
      <div className="flex justify-end">
        <button className="qsae-btn-primary flex items-center gap-2">
          <Download className="w-4 h-4" />
          Download .qsae
        </button>
      </div>
    </div>
  );
};

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

export default Visualizer;
